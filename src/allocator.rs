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
    /// Crée un nouvel allocateur vide (sera initialisé au premier appel).
    pub const fn empty() -> Self {
        Self {
            heap: UnsafeCell::new(Heap {
                start: 0,
                end: 0,
                next: 0,
            }),
        }
    }
    
    /// Initialise l'allocateur avec une zone mémoire.
    /// 
    /// # Safety
    /// - Doit être appelé une seule fois avant toute allocation
    /// - `heap_start` et `heap_size` doivent pointer vers une zone mémoire valide
    unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        let heap = &mut *self.heap.get();
        heap.start = heap_start;
        heap.end = heap_start + heap_size;
        heap.next = heap_start;
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
        
        // Initialisation lazy au premier appel
        if heap.start == 0 {
            self.init(
                core::ptr::addr_of!(HEAP_MEMORY) as usize,
                65536,
            );
        }
        
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
static GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator::empty();

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

    #[test]
    fn test_align_up_power_of_two() {
        assert_eq!(align_up(10, 8), 16);
        assert_eq!(align_up(17, 16), 32);
        assert_eq!(align_up(100, 64), 128);
    }

    #[test]
    fn test_align_up_already_aligned() {
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(32, 8), 32);
        assert_eq!(align_up(64, 32), 64);
    }

    #[test]
    fn test_multiple_allocations() {
        use core::alloc::{GlobalAlloc, Layout};
        let allocator = BumpAllocator::empty();
        
        unsafe {
            // Simule une zone mémoire de test
            allocator.init(0x1000, 1024);
            
            // Première allocation
            let layout = Layout::from_size_align(16, 8).unwrap();
            let ptr1 = allocator.alloc(layout);
            assert!(!ptr1.is_null());
            
            // Deuxième allocation
            let ptr2 = allocator.alloc(layout);
            assert!(!ptr2.is_null());
            
            // Les pointeurs doivent être différents
            assert_ne!(ptr1, ptr2);
        }
    }
}