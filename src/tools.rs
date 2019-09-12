pub use crate::oned::sweeper_update;

#[inline(always)]
pub(crate) fn duplicate_empty_slice<T>(arr: &mut [T]) -> (&mut [T],&mut [T]) {
    assert!(arr.is_empty());
    (unsafe { core::slice::from_raw_parts_mut(arr.as_mut_ptr(), 0) },
    unsafe { core::slice::from_raw_parts_mut(arr.as_mut_ptr(), 0) })
}




use core::marker::PhantomData;
///A Unique that doesnt require rust nightly.
///See https://doc.rust-lang.org/1.26.2/core/ptr/struct.Unique.html
#[derive(Copy,Clone,Debug)]
pub(crate) struct Unique<T: ?Sized>(
    pub core::ptr::NonNull<T>,
    PhantomData<T>
);

unsafe impl<T:?Sized+Send> Send for Unique<T>{}
unsafe impl<T:?Sized+Sync> Sync for Unique<T>{}
impl<T:?Sized> Unique<T>{
    #[inline]
    pub fn new(ptr:*mut T)->Option<Unique<T>>{
        core::ptr::NonNull::new(ptr).map(|a|Unique(a,PhantomData))
    }
    #[inline]
    pub fn as_ptr(&self)->*mut T{
        self.0.as_ptr()
    }
}


use crate::NumTrait;
use alloc::vec::Vec;
use crate::bbox::BBoxRefMut;
//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}



use crate::elem::*;
///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVecMut<T> {
    vec:Vec<*mut T>
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
    pub fn get_empty_vec_mut<'a,'b:'a>(&'a mut self) -> &'a mut Vec<ProtectedBBox<T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe{&mut *(v as *mut _ as *mut Vec<_>)}
    }
}





