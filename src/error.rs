/// Erreurs possibles lors de l'utilisation du parser FAT32.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fat32Error {
    /// Erreur de lecture/écriture sur le périphérique bloc.
    IoError,
    
    /// Tentative d'accès en dehors des limites du périphérique.
    OutOfBounds,
    
    /// Le secteur de boot n'est pas valide (signature incorrecte).
    InvalidBootSector,
    
    /// Le système de fichiers n'est pas un FAT32 valide.
    NotFat32,
    
    /// Cluster invalide (ex: cluster 0 ou 1, ou au-delà de la limite).
    InvalidCluster(u32),
    
    /// Chemin ou nom de fichier invalide.
    InvalidPath,
    
    /// Fichier ou répertoire introuvable.
    NotFound,
    
    /// Tentative d'opération sur un répertoire alors qu'un fichier est attendu.
    IsDirectory,
    
    /// Tentative d'opération sur un fichier alors qu'un répertoire est attendu.
    IsNotDirectory,
    
    /// Buffer trop petit pour contenir les données demandées.
    BufferTooSmall,
}

/// Type Result spécialisé pour le parser FAT32.
pub type Result<T> = core::result::Result<T, Fat32Error>;

impl From<crate::block_device::BlockDeviceError> for Fat32Error {
    fn from(err: crate::block_device::BlockDeviceError) -> Self {
        match err {
            crate::block_device::BlockDeviceError::IoError => Fat32Error::IoError,
            crate::block_device::BlockDeviceError::OutOfBounds => Fat32Error::OutOfBounds,
        }
    }
}

// Pour compatibilité avec core::fmt::Display (utile mais pas obligatoire en no_std)
impl core::fmt::Display for Fat32Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Fat32Error::IoError => write!(f, "I/O error"),
            Fat32Error::OutOfBounds => write!(f, "Out of bounds access"),
            Fat32Error::InvalidBootSector => write!(f, "Invalid boot sector"),
            Fat32Error::NotFat32 => write!(f, "Not a FAT32 filesystem"),
            Fat32Error::InvalidCluster(c) => write!(f, "Invalid cluster: {}", c),
            Fat32Error::InvalidPath => write!(f, "Invalid path"),
            Fat32Error::NotFound => write!(f, "File or directory not found"),
            Fat32Error::IsDirectory => write!(f, "Is a directory"),
            Fat32Error::IsNotDirectory => write!(f, "Not a directory"),
            Fat32Error::BufferTooSmall => write!(f, "Buffer too small"),
        }
    }
}