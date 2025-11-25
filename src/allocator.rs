// On importe les types nécessaires pour écrire un allocateur global
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

// Taille de notre "tas" (heap) en mémoire : ici 1 Mo.
// C'est une zone mémoire dans laquelle on va venir piocher.
const HEAP_SIZE: usize = 1024 * 1024;

// On définit une structure Heap qui sera alignée sur 8 octets.
// L'alignement, c'est pour respecter les contraintes du CPU.
#[repr(align(8))]
struct Heap {
    _inner: [u8; HEAP_SIZE],
}

// On crée une variable globale HEAP qui contient notre mémoire brute.
// `static mut` = global modifiable, donc dangereux si mal utilisé → à encadrer.
static mut HEAP: Heap = Heap {
    _inner: [0; HEAP_SIZE],
};

// Notre allocateur "bump" : il a juste besoin de savoir
// où il en est (`next`) et où il s’arrête (`end`).
pub struct BumpAllocator {
    next: usize,
    end: usize,
}

// On dit au compilateur que cette structure peut être utilisée
// depuis plusieurs threads. Ici, c'est à nous de garantir la sécurité.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    // Constructeur de l'allocateur.
    // Il initialise `next` et `end` pour pointer sur notre HEAP statique.
    //
    // # Safety
    // Cette fonction suppose que HEAP est bien initialisé
    // et qu'on ne fera qu'une seule instance de BumpAllocator.
    pub const unsafe fn new() -> Self {
        // On récupère l'adresse de début du tableau HEAP.
        let start = HEAP._inner.as_ptr() as usize;
        Self {
            next: start,
            end: start + HEAP_SIZE,
        }
    }

    // Fonction interne qui fait vraiment l’allocation.
    //
    // # Safety
    // - Doit être appelée uniquement depuis l'implémentation de GlobalAlloc
    // - Non thread-safe (pas de synchronisation).
    unsafe fn alloc_inner(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        // On part de la prochaine adresse disponible.
        let mut current = self.next;

        // On aligne l'adresse si nécessaire.
        if current % align != 0 {
            current += align - (current % align);
        }

        // Si on dépasse la fin du heap, on n'a plus de mémoire.
        if current + size > self.end {
            return null_mut();
        }

        // On réserve la place en avançant `next`.
        self.next = current + size;

        // On renvoie un pointeur vers la zone allouée.
        current as *mut u8
    }
}

// C’est ici qu’on respecte le contrat de GlobalAlloc.
// C’est ce que le compilateur va appeler quand on utilise Box, Vec, etc.
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Petit hack : on convertit &self (non mutable) en &mut self
        // car l'API GlobalAlloc donne `&self` mais nous avons besoin
        // de modifier l'état interne (next).
        let this = self as *const _ as *mut BumpAllocator;
        (*this).alloc_inner(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator simple : on ne libère jamais.
        // La mémoire sera rendue seulement quand le programme s'arrête.
        // Pour le projet, ce comportement simple est suffisant.
    }
}

// On déclare notre allocateur global pour tout le crate.
// À partir de là, toutes les allocations (Box, Vec, etc.) passeront par lui.
#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = unsafe { BumpAllocator::new() };
