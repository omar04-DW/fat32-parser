// Représente une entrée de répertoire FAT32 brute sur 32 octets.
// Ici on ne gère que les noms courts (8.3), pas les noms longs.
#[repr(C, packed)]
pub struct DirectoryEntryRaw {
    pub name: [u8; 11],
    pub attributes: u8,
    pub reserved: u8,
    pub creation_time_tenth: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_access_date: u16,
    pub first_cluster_high: u16,
    pub write_time: u16,
    pub write_date: u16,
    pub first_cluster_low: u16,
    pub file_size: u32,
}

impl DirectoryEntryRaw {
    // True si l'entrée est libre ou marquée comme supprimée.
    pub fn is_unused(&self) -> bool {
        self.name[0] == 0x00 || self.name[0] == 0xE5
    }

    // True si l'entrée correspond à un dossier.
    pub fn is_dir(&self) -> bool {
        self.attributes & 0x10 != 0
    }

    // Récupère le numéro de premier cluster (high + low).
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16)
            | (self.first_cluster_low as u32)
    }
}