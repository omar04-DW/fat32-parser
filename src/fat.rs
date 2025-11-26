/// Représente une entrée dans la table FAT (File Allocation Table).
/// 
/// En FAT32, chaque entrée fait 32 bits et pointe vers le cluster suivant
/// dans la chaîne, ou contient une valeur spéciale (fin de chaîne, secteur défectueux, etc.).
/// 
/// # Structure de la valeur
/// 
/// - `0x00000000` : cluster libre
/// - `0x00000002..=0x0FFFFFEF` : numéro du cluster suivant
/// - `0x0FFFFFF7` : secteur défectueux (bad cluster)
/// - `0x0FFFFFF8..=0x0FFFFFFF` : fin de chaîne (End Of Chain)
/// 
/// # Exemples
/// 
/// ```
/// use fat32_parser::fat::FatEntry;
/// 
/// let entry = FatEntry { value: 0x0FFFFFF8 };
/// assert!(entry.is_end());
/// 
/// let next_cluster = FatEntry { value: 0x00000003 };
/// assert!(!next_cluster.is_end());
/// assert_eq!(next_cluster.next_cluster(), Some(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FatEntry {
    /// La valeur brute de l'entrée FAT (32 bits).
    pub value: u32,
}

impl FatEntry {
    /// Crée une nouvelle entrée FAT à partir d'une valeur brute.
    /// 
    /// # Exemples
    /// 
    /// ```
    /// use fat32_parser::fat::FatEntry;
    /// 
    /// let entry = FatEntry::new(0x0FFFFFF8);
    /// assert!(entry.is_end());
    /// ```
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    /// Vérifie si cette entrée indique la fin de la chaîne de clusters.
    /// 
    /// Les valeurs >= `0x0FFFFFF8` sont réservées pour marquer la fin d'une chaîne.
    /// 
    /// # Exemples
    /// 
    /// ```
    /// use fat32_parser::fat::FatEntry;
    /// 
    /// assert!(FatEntry::new(0x0FFFFFF8).is_end());
    /// assert!(FatEntry::new(0x0FFFFFFF).is_end());
    /// assert!(!FatEntry::new(0x00000003).is_end());
    /// ```
    pub fn is_end(&self) -> bool {
        self.value >= 0x0FFFFFF8
    }

    /// Vérifie si le cluster est marqué comme libre (disponible).
    /// 
    /// # Exemples
    /// 
    /// ```
    /// use fat32_parser::fat::FatEntry;
    /// 
    /// assert!(FatEntry::new(0x00000000).is_free());
    /// assert!(!FatEntry::new(0x00000003).is_free());
    /// ```
    pub fn is_free(&self) -> bool {
        self.value == 0x00000000
    }

    /// Vérifie si le cluster est marqué comme défectueux (bad cluster).
    /// 
    /// # Exemples
    /// 
    /// ```
    /// use fat32_parser::fat::FatEntry;
    /// 
    /// assert!(FatEntry::new(0x0FFFFFF7).is_bad());
    /// assert!(!FatEntry::new(0x00000003).is_bad());
    /// ```
    pub fn is_bad(&self) -> bool {
        self.value == 0x0FFFFFF7
    }

    /// Retourne le numéro du cluster suivant si l'entrée pointe vers un autre cluster.
    /// 
    /// Retourne `None` si c'est la fin de chaîne, un cluster libre ou défectueux.
    /// 
    /// # Exemples
    /// 
    /// ```
    /// use fat32_parser::fat::FatEntry;
    /// 
    /// assert_eq!(FatEntry::new(0x00000005).next_cluster(), Some(5));
    /// assert_eq!(FatEntry::new(0x0FFFFFF8).next_cluster(), None);
    /// assert_eq!(FatEntry::new(0x00000000).next_cluster(), None);
    /// ```
    pub fn next_cluster(&self) -> Option<u32> {
        if self.is_free() || self.is_bad() || self.is_end() {
            None
        } else {
            Some(self.value & 0x0FFFFFFF)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_of_chain() {
        assert!(FatEntry::new(0x0FFFFFF8).is_end());
        assert!(FatEntry::new(0x0FFFFFFF).is_end());
        assert!(!FatEntry::new(0x0FFFFFF7).is_end());
    }

    #[test]
    fn test_free_cluster() {
        assert!(FatEntry::new(0x00000000).is_free());
        assert!(!FatEntry::new(0x00000001).is_free());
    }

    #[test]
    fn test_bad_cluster() {
        assert!(FatEntry::new(0x0FFFFFF7).is_bad());
        assert!(!FatEntry::new(0x0FFFFFF8).is_bad());
    }

    #[test]
    fn test_next_cluster() {
        assert_eq!(FatEntry::new(0x00000003).next_cluster(), Some(3));
        assert_eq!(FatEntry::new(0x0FFFFFF8).next_cluster(), None);
        assert_eq!(FatEntry::new(0x00000000).next_cluster(), None);
    }
}