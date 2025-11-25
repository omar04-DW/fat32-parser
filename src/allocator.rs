// Allocateur global ultra simple pour le projet.
//
// IMPORTANT :
// Cet allocateur ne fait PAS de vraie allocation.
// Il existe surtout pour respecter la contrainte de l'examen :
//   - projet no_std
//   - implémentation d'un GlobalAlloc personnalisé
//
// C'est exactement l'étape dont parle le cours : "notre allocateur
// est appelé mais renvoie toujours un pointeur nul, il faudra
// l'améliorer plus tard".
//
// Tant que tu n'utilises pas Box / Vec / etc. dans le code,
// ce comportement ne posera pas de problème.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// Allocateur "dummy" : il ne sait rien allouer et renvoie toujours null.
pub struct DummyAllocator;

// On dit au compilateur que notre allocateur peut être utilisé de manière
// thread-safe. Ici c'est safe car il n'a pas d'état interne.
unsafe impl Sync for DummyAllocator {}

unsafe impl GlobalAlloc for DummyAllocator {
    /// Fonction appelée quand le code essaie d'allouer de la mémoire
    /// (par exemple avec Box::new, Vec::new, etc.).
    ///
    /// Ici, on ne fait rien et on renvoie juste un pointeur nul.
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // Pointeur nul = allocation échouée.
        // Dans un vrai projet, on implémenterait un vrai heap ici.
        null_mut()
    }

    /// Fonction appelée quand on libère de la mémoire.
    /// Comme on ne fait pas d'allocation, on ignore simplement l'appel.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Rien à faire : notre allocateur n'a jamais vraiment alloué.
    }
}

// On enregistre notre allocateur "dummy" comme allocateur global
// pour tout le crate.
//
// À partir de maintenant, si le code essaie d'allouer dynamiquement,
// ce sera ce DummyAllocator qui sera utilisé.
#[global_allocator]
static GLOBAL_ALLOCATOR: DummyAllocator = DummyAllocator;