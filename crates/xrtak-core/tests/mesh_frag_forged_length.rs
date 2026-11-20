use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use xrtak_core::mesh_frag::{FragmentReassemblyBuffer, MeshFragment};

static LARGEST: AtomicUsize = AtomicUsize::new(0);
struct Probe;
unsafe impl GlobalAlloc for Probe {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static PROBE: Probe = Probe;

#[test]
fn forged_header_does_not_reserve_the_declared_length() {
    let wire = [0, 0, 0, 1, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF];
    let fragment = MeshFragment::from_bytes(&wire).unwrap();
    let mut buffer = FragmentReassemblyBuffer::new(&fragment.header, 0);
    assert!(buffer.insert_fragment(fragment, 0));
    assert_eq!(buffer.try_reassemble(), None);
    let largest = LARGEST.load(Ordering::Relaxed);
    assert!(largest < 1 << 20, "a 12-byte datagram reserved {largest} bytes");
}
