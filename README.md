FAT32 Parser

Parser FAT32 no_std pour systèmes embarqués en Rust.

✅ Fonctionnalités

- ✅ Parsing du boot sector FAT32 (signature 0x55AA)
- ✅ Lecture de la BPB (BIOS Parameter Block)
- ✅ Lecture de la table FAT
- ✅ Navigation dans les répertoires
- ✅ Lecture des chaînes de clusters
- ✅ Allocateur Bump (64KB)
- ✅ Compatible no_std
- ✅ 22 tests (14 unit + 8 doc)
- ✅ Documentation complète

🏗️ Structure

<img width="453" height="299" alt="image" src="https://github.com/user-attachments/assets/e8dafba2-5223-45db-a15c-e3b5d8732fe0" />


### Détails des modules

**lib.rs** 
- Configuration `no_std`
- Exports publics des modules
- Point d'entrée de la bibliothèque

**allocator.rs** 
- Structure `BumpAllocator` avec heap statique 64KB
- Initialisation lazy au premier `alloc()`
- Fonction `align_up()` pour l'alignement mémoire
- 4 tests unitaires

**block_device.rs** 
- Enum `BlockDeviceError` (IoError, OutOfBounds)
- Trait `BlockDevice` avec constante SECTOR_SIZE
- Méthode `read_sectors()` abstraite
- 1 test unitaire

**boot_sector.rs** 
- Structure `BiosParameterBlock` (#[repr(C, packed)])
- Structure `Fat32Geometry` pour calculs d'adresses
- Fonction `cluster_to_lba()` pour conversion
- Documentation Safety complète
- 1 test unitaire + 1 doctest

**dir_entry.rs** 
- Structure `DirectoryEntryRaw` de 32 bytes
- Méthodes `is_unused()`, `is_dir()`, `first_cluster()`
- Support noms courts 8.3 uniquement

**fat.rs** 
- Structure `FatEntry` pour entrées 32 bits
- Méthodes `is_end()`, `is_free()`, `is_bad()`, `next_cluster()`
- Détection EOC (≥ 0x0FFFFFF8)
- 4 tests unitaires + 4 doctests

**filesystem.rs** 
- Structure `Fat32Fs<'a, D>` pour le FS monté
- Méthode `mount()` avec validation boot sector
- Méthode `read_fat_entry()` pour lecture table FAT
- Méthode `read_cluster()` pour lecture individuelle
- Méthode `read_cluster_chain()` avec callback
- Structure `DirectoryIterator` pour parcours
- 4 tests unitaires + 1 doctest

**error.rs** 
- Enum `Fat32Error` avec 10 variantes
- Type alias `Result<T>`
- Conversion `From<BlockDeviceError>`
- Implémentation `Display` pour messages

### Fichiers de configuration

**Cargo.toml**
- Métadonnées du projet (nom, version, auteur)
- Aucune dépendance externe (no_std)
- Profils de compilation (dev, release, test)

**rust-toolchain.toml**
- Channel: `nightly-2024-11-01`
- Components: rustfmt, clippy, rust-src
- Target: x86_64-unknown-none

🚀 Compilation

# Build
cargo build --release

# Tests
cargo test

# Documentation
cargo doc --open

# Linting
cargo clippy

📊 Tests
```
running 14 tests (unit tests)
✓ allocator::test_align_up
✓ allocator::test_align_up_power_of_two
✓ allocator::test_align_up_already_aligned
✓ allocator::test_multiple_allocations
✓ block_device::test_sector_size
✓ boot_sector::test_cluster_to_lba
✓ fat::test_end_of_chain
✓ fat::test_free_cluster
✓ fat::test_bad_cluster
✓ fat::test_next_cluster
✓ filesystem::test_invalid_boot_sector
✓ filesystem::test_boot_sector_too_small
✓ filesystem::test_valid_signature_but_not_fat32
✓ filesystem::test_fat_entry_reading

running 8 tests (doc tests)
✓ All doctests passed

test result: ok. 22 passed; 0 failed

⚙️ Détails techniques

### Allocateur
- Heap statique de 64KB
- Initialisation lazy
- Pas de libération individuelle

### Parser FAT32
- Vérification signature boot sector
- Lecture entrées FAT (32 bits)
- Détection EOC (≥ 0x0FFFFFF8)
- Protection boucles infinies (100k clusters max)
