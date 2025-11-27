use crate::block_device::BlockDevice;
use crate::boot_sector::{BiosParameterBlock, Fat32Geometry};
use crate::error::{Fat32Error, Result};
use crate::fat::FatEntry;
use crate::dir_entry::DirectoryEntryRaw;

/// Représente un système de fichiers FAT32 monté sur un périphérique bloc.
/// 
/// # Exemples
/// 
/// ```no_run
/// use fat32_parser::{Fat32Fs, BlockDevice};
/// 
/// fn mount_example<D: BlockDevice>(device: &D) -> Result<(), fat32_parser::error::Fat32Error> {
///     let mut boot_sector = [0u8; 512];
///     device.read_sectors(0, 1, &mut boot_sector)?;
///     
///     let fs = Fat32Fs::mount(device, &boot_sector)?;
///     // Utiliser le système de fichiers...
///     Ok(())
/// }
/// ```
pub struct Fat32Fs<'a, D: BlockDevice> {
    pub device: &'a D,
    pub geom: Fat32Geometry,
}

impl<'a, D: BlockDevice> Fat32Fs<'a, D> {
    /// Crée une nouvelle instance avec un périphérique et une géométrie donnés.
    pub fn new(device: &'a D, geom: Fat32Geometry) -> Self {
        Self { device, geom }
    }

    /// Monte un volume FAT32 à partir du secteur de boot.
    /// 
    /// # Errors
    /// 
    /// Retourne une erreur si le secteur de boot n'est pas valide.
    /// 
    /// # Safety
    /// 
    /// Cette fonction utilise du code unsafe pour caster les octets bruts
    /// en structure BPB. Le secteur fourni doit contenir un boot sector valide.
    pub fn mount(device: &'a D, boot_sector: &[u8]) -> Result<Self> {
        // Vérifie la signature du boot sector (octets 510-511 = 0x55AA)
        if boot_sector.len() < 512 {
            return Err(Fat32Error::InvalidBootSector);
        }
        
        if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
            return Err(Fat32Error::InvalidBootSector);
        }

        // SAFETY: On a vérifié que boot_sector fait au moins 512 octets
        // et contient la signature valide
        let bpb = unsafe { BiosParameterBlock::from_sector(boot_sector) };
        
        // Vérifie que c'est bien FAT32 (fat_size_16 doit être 0)
        if bpb.fat_size_16 != 0 || bpb.fat_size_32 == 0 {
            return Err(Fat32Error::NotFat32);
        }

        let geom = Fat32Geometry::from_bpb(bpb);
        Ok(Fat32Fs::new(device, geom))
    }

    /// Lit une entrée de la table FAT.
    /// 
    /// # Arguments
    /// 
    /// * `cluster` - Numéro du cluster dont on veut lire l'entrée FAT
    /// 
    /// # Errors
    /// 
    /// Retourne une erreur si la lecture échoue ou si le cluster est invalide.
    pub fn read_fat_entry(&self, cluster: u32) -> Result<FatEntry> {
        if cluster < 2 {
            return Err(Fat32Error::InvalidCluster(cluster));
        }

        // Calcul de l'offset dans la FAT (4 octets par entrée en FAT32)
        let fat_offset = cluster * 4;
        let fat_sector = self.geom.fat_start_lba + (fat_offset / self.geom.bytes_per_sector);
        let entry_offset = (fat_offset % self.geom.bytes_per_sector) as usize;

        // Lit le secteur contenant l'entrée FAT
        let mut sector = [0u8; 512];
        self.device
            .read_sectors(fat_sector, 1, &mut sector)
            .map_err(Fat32Error::from)?;

        // Extrait la valeur 32 bits (little-endian)
        let value = u32::from_le_bytes([
            sector[entry_offset],
            sector[entry_offset + 1],
            sector[entry_offset + 2],
            sector[entry_offset + 3],
        ]);

        Ok(FatEntry::new(value & 0x0FFFFFFF)) // Masque les 4 bits de poids fort
    }

    /// Lit un cluster entier dans un buffer.
    /// 
    /// # Arguments
    /// 
    /// * `cluster` - Numéro du cluster à lire
    /// * `buf` - Buffer de destination (doit être >= taille_cluster)
    /// 
    /// # Errors
    /// 
    /// Retourne une erreur si le buffer est trop petit ou si la lecture échoue.
    pub fn read_cluster(&self, cluster: u32, buf: &mut [u8]) -> Result<()> {
        if cluster < 2 {
            return Err(Fat32Error::InvalidCluster(cluster));
        }

        let cluster_size = (self.geom.sectors_per_cluster * self.geom.bytes_per_sector) as usize;
        if buf.len() < cluster_size {
            return Err(Fat32Error::BufferTooSmall);
        }

        let lba = self.geom.cluster_to_lba(cluster);
        self.device
            .read_sectors(lba, self.geom.sectors_per_cluster, buf)
            .map_err(Fat32Error::from)
    }

    /// Lit la chaîne complète de clusters (utile pour lire un fichier entier).
    /// 
    /// # Arguments
    /// 
    /// * `start_cluster` - Premier cluster de la chaîne
    /// * `callback` - Fonction appelée pour chaque cluster lu
    /// 
    /// # Errors
    /// 
    /// Retourne une erreur si la lecture échoue.
    pub fn read_cluster_chain<F>(&self, start_cluster: u32, mut callback: F) -> Result<()>
    where
        F: FnMut(u32, &[u8]) -> Result<()>,
    {
        let cluster_size = (self.geom.sectors_per_cluster * self.geom.bytes_per_sector) as usize;
        let mut buf = [0u8; 4096]; // Suppose cluster <= 4KB
        
        if cluster_size > buf.len() {
            return Err(Fat32Error::BufferTooSmall);
        }

        let mut current_cluster = start_cluster;
        let mut cluster_count = 0;
        const MAX_CLUSTERS: u32 = 100000; // Protection contre boucles infinies

        loop {
            // Protection contre boucles infinies
            if cluster_count >= MAX_CLUSTERS {
                return Err(Fat32Error::InvalidCluster(current_cluster));
            }

            // Lit le cluster
            self.read_cluster(current_cluster, &mut buf[..cluster_size])?;
            callback(current_cluster, &buf[..cluster_size])?;

            // Lit l'entrée FAT pour trouver le cluster suivant
            let fat_entry = self.read_fat_entry(current_cluster)?;
            
            if fat_entry.is_end() {
                break;
            }

            match fat_entry.next_cluster() {
                Some(next) => current_cluster = next,
                None => break,
            }

            cluster_count += 1;
        }

        Ok(())
    }

    /// Lit le répertoire racine.
    /// 
    /// # Returns
    /// 
    /// Un itérateur sur les entrées de répertoire du root.
    pub fn read_root_dir(&self) -> Result<DirectoryIterator<'_, 'a, D>> {
        DirectoryIterator::new(self, self.geom.root_cluster)
    }
}

/// Itérateur sur les entrées d'un répertoire FAT32.
pub struct DirectoryIterator<'fs, 'a, D: BlockDevice> {
    fs: &'fs Fat32Fs<'a, D>,
    cluster: u32,
    offset: usize,
    buffer: [u8; 4096],
    done: bool,
}

impl<'fs, 'a, D: BlockDevice> DirectoryIterator<'fs, 'a, D> {
    fn new(fs: &'fs Fat32Fs<'a, D>, start_cluster: u32) -> Result<Self> {
        let mut iter = Self {
            fs,
            cluster: start_cluster,
            offset: 0,
            buffer: [0u8; 4096],
            done: false,
        };
        
        // Charge le premier cluster
        let cluster_size = (fs.geom.sectors_per_cluster * fs.geom.bytes_per_sector) as usize;
        fs.read_cluster(start_cluster, &mut iter.buffer[..cluster_size])?;
        
        Ok(iter)
    }

    /// Retourne la prochaine entrée de répertoire valide.
    pub fn next_entry(&mut self) -> Result<Option<&DirectoryEntryRaw>> {
        if self.done {
            return Ok(None);
        }

        loop {
            // Vérifie si on est à la fin du cluster actuel
            if self.offset >= 4096 {
                // Charge le cluster suivant
                let fat_entry = self.fs.read_fat_entry(self.cluster)?;
                
                if fat_entry.is_end() {
                    self.done = true;
                    return Ok(None);
                }

                match fat_entry.next_cluster() {
                    Some(next) => {
                        self.cluster = next;
                        self.offset = 0;
                        let cluster_size = (self.fs.geom.sectors_per_cluster 
                                          * self.fs.geom.bytes_per_sector) as usize;
                        self.fs.read_cluster(next, &mut self.buffer[..cluster_size])?;
                    }
                    None => {
                        self.done = true;
                        return Ok(None);
                    }
                }
            }

            // SAFETY: buffer contient des données valides alignées sur 32 octets
            let entry = unsafe {
                &*(self.buffer.as_ptr().add(self.offset) as *const DirectoryEntryRaw)
            };

            self.offset += 32; // Taille d'une entrée de répertoire

            // Fin du répertoire
            if entry.name[0] == 0x00 {
                self.done = true;
                return Ok(None);
            }

            // Entrée supprimée ou volume label, on skip
            if entry.is_unused() || entry.attributes == 0x08 {
                continue;
            }

            return Ok(Some(entry));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_device::BlockDeviceError;

    struct DummyDevice;

    impl BlockDevice for DummyDevice {
        fn read_sectors(
            &self,
            _lba: u32,
            _count: u32,
            _buf: &mut [u8],
        ) -> core::result::Result<(), BlockDeviceError> {
            Ok(())
        }
    }

    #[test]
    fn test_invalid_boot_sector() {
        let dev = DummyDevice;
        let invalid_boot = [0u8; 512]; // Signature invalide
        assert!(matches!(
            Fat32Fs::mount(&dev, &invalid_boot),
            Err(Fat32Error::InvalidBootSector)
        ));
    }

    #[test]
    fn test_boot_sector_too_small() {
        let dev = DummyDevice;
        let small_boot = [0u8; 100]; // Trop petit
        assert!(matches!(
            Fat32Fs::mount(&dev, &small_boot),
            Err(Fat32Error::InvalidBootSector)
        ));
    }
        #[test]
    fn test_valid_signature_but_not_fat32() {
        let dev = DummyDevice;
        let mut boot = [0u8; 512];
        boot[510] = 0x55;
        boot[511] = 0xAA;
        // Signature valide mais BPB invalide (fat_size_32 = 0)
        let result = Fat32Fs::mount(&dev, &boot);
        assert!(matches!(result, Err(Fat32Error::NotFat32)));
    }

    #[test]
    fn test_fat_entry_reading() {
        // Test avec un device qui retourne des données connues
    }
}