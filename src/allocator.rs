use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

/// Allocateur simple de type "bump" : il avance un pointeur à chaque allocation.
/// ATTENTION : ne libère jamais la mémoire (dealloc ne fait rien).
pub struct BumpAllocator {
    heap: UnsafeCell<Heap>,
}

struct Heap {
    start: usize,
    end: usize,
    next: usize,
}

impl BumpAllocator {
    /// Crée un nouvel allocateur sur une zone mémoire donnée.
    /// 
    /// # Safety
    /// - `heap_start` et `heap_size` doivent pointer vers une zone mémoire valide
    /// - Cette zone ne doit pas être utilisée ailleurs
    /// - Doit être appelé une seule fois
    pub const unsafe fn new(heap_start: usize, heap_size: usize) -> Self {
        Self {
            heap: UnsafeCell::new(Heap {
                start: heap_start,
                end: heap_start + heap_size,
                next: heap_start,
            }),
        }
    }
}

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    /// Alloue de la mémoire en avançant le pointeur.
    /// 
    /// # Safety
    /// Retourne null si pas assez de mémoire disponible.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = &mut *self.heap.get();
        
        // Aligne le pointeur selon les besoins du Layout
        let alloc_start = align_up(heap.next, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());

        // Vérifie qu'on ne dépasse pas la fin du heap
        if alloc_end > heap.end {
            return ptr::null_mut();
        }

        // Avance le pointeur pour la prochaine allocation
        heap.next = alloc_end;
        
        alloc_start as *mut u8
    }

    /// Libère la mémoire (ne fait rien dans un bump allocator).
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Un bump allocator ne libère pas individuellement
        // La mémoire est libérée en une fois quand tout le heap est réinitialisé
    }
}

/// Arrondit `addr` au multiple supérieur de `align`.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// Zone de heap statique (pour la démo - 64KB)
static mut HEAP_MEMORY: [u8; 65536] = [0; 65536];

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: BumpAllocator = unsafe {
    BumpAllocator::new(
        HEAP_MEMORY.as_ptr() as usize,
        HEAP_MEMORY.len(),
    )
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
    }
}