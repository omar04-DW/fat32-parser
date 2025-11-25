// Les différentes erreurs possibles quand on lit sur un "disque" logique.
// Ici on ne gère que quelques cas simples pour le projet.
#[derive(Debug)]
pub enum BlockDeviceError {
    IoError,
    OutOfBounds,
}

// Ce trait représente un support de stockage type "bloc" (block device).
// Exemple : un disque, une image disque, une carte SD, etc.
pub trait BlockDevice {
    // Taille d'un secteur en octets.
    // On met 512 par défaut, c'est la valeur classique en FAT32.
    const SECTOR_SIZE: usize = 512;

    // Fonction de lecture : on lit `count` secteurs à partir du LBA `lba`
    // et on écrit les données dans `buf`.
    fn read_sectors(
        &self,
        lba: u32,
        count: u32,
        buf: &mut [u8],
    ) -> Result<(), BlockDeviceError>;
}

// Petit module de tests basiques pour vérifier que notre trait tient la route.
#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    // Un device fictif qui ne fait rien mais implémente BlockDevice,
    // juste pour tester que le code compile et fonctionne.
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

    // On vérifie juste que la constante SECTOR_SIZE vaut 512.
    #[test]
    fn test_sector_size() {
        assert_eq!(DummyDevice::SECTOR_SIZE, 512);
    }
}