//! A global allocator that counts what the calling thread allocates, so a
//! test can assert how many times a hot path allocates rather than guess
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! It counts per thread, so tests running in parallel do not see each
//! other's allocations, and it counts nothing until a measurement asks it
//! to, so the allocator costs a branch when it is not being used.
//!
//! ```
//! use teistro_test_allocator::{Counting, measure};
//!
//! #[global_allocator]
//! static ALLOCATOR: Counting = Counting::system();
//!
//! let (sum, counts) = measure(|| (1..=10).sum::<u32>());
//! assert_eq!(sum, 55);
//! assert_eq!(counts.allocations, 0, "arithmetic allocates nothing");
//!
//! let (text, counts) = measure(|| String::from("one allocation"));
//! assert_eq!(text.len(), 14);
//! assert_eq!(counts.allocations, 1);
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// What this thread has allocated since a measurement began, and
    /// whether one is running. `const` initialisation, because a lazy one
    /// would allocate inside the allocator.
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
    static BYTES: Cell<u64> = const { Cell::new(0) };
    static MEASURING: Cell<bool> = const { Cell::new(false) };
}

/// What a measurement counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// How many times the thread asked for memory, reallocations
    /// included: a `Vec` that grows five times allocated five times.
    pub allocations: u64,
    /// How many bytes those requests asked for.
    pub bytes: u64,
}

/// A global allocator that counts the calling thread's allocations while
/// a [`measure`] is running, and hands every request to the one it wraps.
#[derive(Debug)]
pub struct Counting<A = System> {
    /// The allocator every request is handed to.
    inner: A,
}

impl Counting<System> {
    /// The system allocator, counted.
    #[must_use]
    pub const fn system() -> Counting<System> {
        Counting { inner: System }
    }
}

impl<A> Counting<A> {
    /// Another allocator, counted.
    #[must_use]
    pub const fn wrapping(inner: A) -> Counting<A> {
        Counting { inner }
    }
}

/// Records one request, when a measurement is running.
fn record(size: usize) {
    if MEASURING.with(Cell::get) {
        ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
        BYTES.with(|bytes| bytes.set(bytes.get().saturating_add(size as u64)));
    }
}

// SAFETY: every method hands the request to the wrapped allocator
// unchanged and only counts alongside it; the counters are thread-local
// `Cell`s with const initialisation, so no method allocates.
#[allow(
    unsafe_code,
    reason = "a global allocator is an unsafe trait; every method forwards"
)]
unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        // SAFETY: the caller's contract, forwarded.
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller's contract, forwarded.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        // SAFETY: the caller's contract, forwarded.
        unsafe { self.inner.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(new_size);
        // SAFETY: the caller's contract, forwarded.
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }
}

/// Runs `f` and reports what the calling thread allocated while it ran.
///
/// Measurements do not nest: an inner one counts into its own totals and
/// the outer one resumes where it left off, which is what a helper that
/// measures inside a measured block would want.
pub fn measure<T>(f: impl FnOnce() -> T) -> (T, Counts) {
    let outer = (
        MEASURING.with(Cell::get),
        ALLOCATIONS.with(Cell::get),
        BYTES.with(Cell::get),
    );
    ALLOCATIONS.with(|count| count.set(0));
    BYTES.with(|bytes| bytes.set(0));
    MEASURING.with(|on| on.set(true));
    let value = f();
    let counts = Counts {
        allocations: ALLOCATIONS.with(Cell::get),
        bytes: BYTES.with(Cell::get),
    };
    MEASURING.with(|on| on.set(outer.0));
    ALLOCATIONS.with(|count| count.set(outer.1.saturating_add(counts.allocations)));
    BYTES.with(|bytes| bytes.set(outer.2.saturating_add(counts.bytes)));
    (value, counts)
}
