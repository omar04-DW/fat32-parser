// On importe le trait BlockDevice et les structures liées au boot sector.
use crate::block_device::BlockDevice;
use crate::boot_sector::{BiosParameterBlock, Fat32Geometry};

// Cette structure représente un système de fichiers FAT32 "monté"
// sur un BlockDevice quelconque (disque, image, etc.).
pub struct Fat32Fs<'a, D: BlockDevice> {
    pub device: &'a D,
    pub geom: Fat32Geometry,
}

impl<'a, D: BlockDevice> Fat32Fs<'a, D> {
    // Crée une nouvelle instance à partir d'un device et d'une géométrie.
    // Utile si on a déjà calculé la géométrie.
    pub fn new(device: &'a D, geom: Fat32Geometry) -> Self {
        Self { device, geom }
    }

    // Monte un volume FAT32 à partir d'un secteur de boot brut.
    //
    // Ici, on ne fait qu'extraire la BPB puis calculer la géométrie.
    pub fn mount(device: &'a D, boot_sector: &[u8]) -> Self {
        // Appel à une fonction unsafe car on caste des octets en structure.
        // On suppose que `boot_sector` contient vraiment un boot sector FAT32.
        let bpb = unsafe { BiosParameterBlock::from_sector(boot_sector) };
        let geom = Fat32Geometry::from_bpb(bpb);
        Fat32Fs::new(device, geom)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::block_device::{BlockDevice, BlockDeviceError};

    // Device d'exemple qui ne fait rien, utilisé pour tester
    // la création d'un Fat32Fs.
    struct DummyDevice;

    impl BlockDevice for DummyDevice {
        fn read_sectors(
            &self,
            _lba: u32,
            _count: u32,
            _buf: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            Ok(())
        }
    }

    // Test : on vérifie qu'on peut créer un Fat32Fs sans panic
    // à partir d'un "faux" secteur de boot rempli de zéros.
    #[test]
    fn fs_creation_works() {
        let dev = DummyDevice;
        let dummy_boot = [0u8; 512];
        let _ = Fat32Fs::mount(&dev, &dummy_boot);
    }
}