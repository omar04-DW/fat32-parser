// On importe les outils nécessaires pour écrire un allocateur global en no_std
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

// On crée une zone mémoire fixe de 1 Mo qui servira de "heap".
// C'est comme si on réservait un grand tableau de bytes pour nos allocations.
const HEAP_SIZE: usize = 1024 * 1024;

// Cette structure représente notre heap mais avec un alignement de 8 octets.
// L’alignement est important car certaines données en mémoire doivent être alignées.
#[repr(align(8))]
struct Heap {
    _inner: [u8; HEAP_SIZE],
}

// On crée un vrai HEAP global qui contient notre mémoire brute.
// Comme c’est global + modifiable → `static mut` (donc dangereux).
// C’est pour ça qu’on entoure tout le reste d’unsafe.
static mut HEAP: Heap = Heap {
    _inner: [0; HEAP_SIZE],
};

// Notre allocateur "Bump" est le plus simple possible :
// - On part du début du heap
// - À chaque allocation, on avance un pointeur (next)
// - On ne revient jamais en arrière (pas de free)
pub struct BumpAllocator {
    next: usize, // prochaine adresse libre
    end: usize,  // fin du heap
}

// On indique que notre allocateur peut être utilisé de manière thread-safe.
// Ici c’est à nous d’assurer la sécurité car on utilise du unsafe.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Constructeur NON-CONST (pour éviter l’erreur du compilateur).
    ///
    /// # Safety
    /// Cette fonction doit être appelée UNE SEULE FOIS.
    pub unsafe fn new() -> Self {
        // Adresse de début du tableau HEAP (notre zone mémoire)
        let start = HEAP._inner.as_ptr() as usize;

        Self {
            next: start,
            end: start + HEAP_SIZE,
        }
    }

    /// Fonction interne qui fait vraiment l’allocation.
    ///
    /// # Safety
    /// - Doit être appelée depuis GlobalAlloc
    /// - Pas sécurisée pour plusieurs threads
    unsafe fn alloc_inner(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align(); // alignement demandé
        let size = layout.size();   // taille demandée

        let mut current = self.next;

        // Si l'adresse actuelle n'est pas alignée, on la décale
        if current % align != 0 {
            current += align - (current % align);
        }

        // Si on n’a plus assez de place, on renvoie null
        if current + size > self.end {
            return null_mut();
        }

        // On réserve la zone
        self.next = current + size;

        // Et on renvoie un pointeur vers la mémoire
        current as *mut u8
    }
}

// Ici, on implémente le trait GlobalAlloc.
// C’est ce que Rust utilise pour allouer la mémoire avec Box, Vec, etc.
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Hack obligatoire : GlobalAlloc ne nous donne qu’un &self,
        // mais on a besoin de &mut self → on cast le pointeur.
        let this = self as *const _ as *mut BumpAllocator;
        (*this).alloc_inner(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator simple : on NE LIBÈRE JAMAIS la mémoire.
        // C'est volontaire → les free sont ignorés.
    }
}

// On dit à Rust : “utilise mon allocateur comme allocateur global du crate”.
#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = unsafe { BumpAllocator::new() };