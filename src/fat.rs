// Représente une entrée de la table FAT (32 bits en FAT32).
pub struct FatEntry {
    pub value: u32,
}

impl FatEntry {
    // Vérifie si cette entrée indique la fin de la chaîne de clusters.
    // Les valeurs >= 0x0FFFFFF8 sont réservées pour "end of chain".
    pub fn is_end(&self) -> bool {
        self.value >= 0x0FFFFFF8
    }
}j