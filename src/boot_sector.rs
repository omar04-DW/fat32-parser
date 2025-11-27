// Structure qui représente la partie importante de la BPB (BIOS Parameter Block)
// d'un volume FAT32. Les champs correspondent à ce qui est défini dans la doc FAT32.
//
// #[repr(C, packed)] signifie :
// - "C" : même ordre et alignement qu'en C
// - "packed" : pas de padding entre les champs (collés)
#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
}

impl BiosParameterBlock {
    /// Construit une référence vers une BPB à partir des octets du secteur de boot.
    ///
    /// # Safety
    ///
    /// Cette fonction est unsafe car elle effectue un cast de pointeur brut sans validation.
    /// L'appelant doit garantir que :
    /// - `sector` contient au moins `11 + size_of::<BiosParameterBlock>()` octets (≈ 47 octets minimum)
    /// - Les octets à partir de l'offset 11 sont correctement alignés pour `BiosParameterBlock`
    /// - Les données représentent une BPB FAT32 valide provenant d'un vrai boot sector
    /// - La durée de vie de `sector` couvre toute utilisation de la référence retournée
    ///
    /// # Exemples
    ///
    /// ```no_run
    /// use fat32_parser::boot_sector::BiosParameterBlock;
    ///
    /// let boot_sector = [0u8; 512]; // Secteur lu depuis un disque
    /// let bpb = unsafe { BiosParameterBlock::from_sector(&boot_sector) };
    /// println!("Bytes per sector: {}", bpb.bytes_per_sector);
    /// ```
    pub unsafe fn from_sector(sector: &[u8]) -> &Self {
        // Dans le format FAT, la BPB commence à l'offset 11 dans le secteur.
        let offset = 11;
        &*(sector.as_ptr().add(offset) as *const BiosParameterBlock)
    }
}

// Structure plus "haut niveau" qui regroupe les infos utiles pour faire
// des calculs d'adresses (clusters → secteurs).
pub struct Fat32Geometry {
    pub first_data_sector: u32,
    pub fat_start_lba: u32,
    pub root_cluster: u32,
    pub sectors_per_cluster: u32,
    pub bytes_per_sector: u32,
}

impl Fat32Geometry {
    // Construit la géométrie à partir de la BPB brute.
    pub fn from_bpb(bpb: &BiosParameterBlock) -> Self {
        let fats = bpb.num_fats as u32;
        let reserved = bpb.reserved_sector_count as u32;

        // Taille d'une FAT en secteurs.
        // En FAT32, fat_size_32 est utilisé, mais on gère aussi le cas 16 bits.
        let fat_size = if bpb.fat_size_16 != 0 {
            bpb.fat_size_16 as u32
        } else {
            bpb.fat_size_32
        };

        // Premier secteur de la zone de données (après les FATs).
        let first_data_sector = reserved + fats * fat_size;

        Self {
            first_data_sector,
            fat_start_lba: reserved,
            root_cluster: bpb.root_cluster,
            sectors_per_cluster: bpb.sectors_per_cluster as u32,
            bytes_per_sector: bpb.bytes_per_sector as u32,
        }
    }

    // Traduit un numéro de cluster FAT en adresse LBA (numéro de secteur logique).
    //
    // Dans FAT32, les clusters commencent à 2.
    pub fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.first_data_sector + (cluster - 2) * self.sectors_per_cluster
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_to_lba() {
        let geom = Fat32Geometry {
            first_data_sector: 100,
            fat_start_lba: 32,
            root_cluster: 2,
            sectors_per_cluster: 8,
            bytes_per_sector: 512,
        };
        
        // Cluster 2 devrait être au premier secteur de données
        assert_eq!(geom.cluster_to_lba(2), 100);
        // Cluster 3 devrait être 8 secteurs plus loin
        assert_eq!(geom.cluster_to_lba(3), 108);
        // Cluster 10
        assert_eq!(geom.cluster_to_lba(10), 164);
    }
}