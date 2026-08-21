#[cfg(all(feature = "alloc_llff", not(feature = "alloc_tlsf")))]
use embedded_alloc::LlffHeap as Heap;
#[cfg(feature = "alloc_tlsf")]
use embedded_alloc::TlsfHeap as Heap;

#[global_allocator]
pub(crate) static ALLOCATOR: Heap = Heap::empty();
