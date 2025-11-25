use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// Allocateur "dummy" : il ne sait rien allouer et renvoie toujours null.
pub struct DummyAllocator;

// Pas d'état interne → OK pour plusieurs threads.
unsafe impl Sync for DummyAllocator {}

unsafe impl GlobalAlloc for DummyAllocator {
    /// Appelé quand on essaie d'allouer de la mémoire (Box, Vec, etc.)
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // On fait semblant d'échouer systématiquement.
        null_mut()
    }

    /// Appelé quand on libère de la mémoire.
    /// Comme on n'alloue jamais vraiment, on ignore.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Rien à faire ici.
    }
}

// IMPORTANT : on active cet allocateur global uniquement
// en build normal (no_std). En mode test, on ne le met pas,
// sinon le runtime de test crashe en essayant d'allouer.
#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: DummyAllocator = DummyAllocator; 