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





use alloc::vec::Vec;



use crate::elem::*;

//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVecMut<T> {
    vec:Vec<core::ptr::NonNull<T>>
}

impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        PreVecMut {
            vec:Vec::new()
        }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a,'b:'a>(&'a mut self) -> &'a mut Vec<ProtectedBBox<'b,T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe{&mut *(v as *mut _ as *mut Vec<_>)}
    }    
}


