pub use crate::oned::sweeper_update;



///Create a duplicate empty slice, provided the input slice is empty
///with the same exact pointer.
///This can make checking certain invariants easier.
#[inline(always)]
pub fn duplicate_empty_slice<T>(arr: &mut [T]) -> (&mut [T],&mut [T]) {
    assert!(arr.is_empty());
    (unsafe { core::slice::from_raw_parts_mut(arr.as_mut_ptr(), 0) },
    unsafe { core::slice::from_raw_parts_mut(arr.as_mut_ptr(), 0) })
}




