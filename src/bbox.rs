use crate::inner_prelude::*;

///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user
///cannot modify the bounding box since it is hidden, with only read access.
#[repr(C)]
pub struct BBoxRef<'a,N:NumTrait,T> {
    pub rect: &'a axgeom::Rect<N>,
    pub inner: &'a T,
}

impl<'a,N:NumTrait,T> BBoxRef<'a, N,T> {
    
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub fn new(rect: &'a axgeom::Rect<N>, inner: &'a T) -> BBoxRef<'a,N,T> {
        BBoxRef{ rect, inner }
    }

}


///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user
///cannot modify the bounding box since it is hidden, with only read access.
#[repr(C)]
pub struct BBoxRefMut<'a,N:NumTrait,T> {
    pub rect: &'a axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N:NumTrait,T> BBoxRefMut<'a,N,T> {
    
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub fn new(rect: &'a axgeom::Rect<N>, inner: &'a mut T) -> BBoxRefMut<'a,N,T> {
        BBoxRefMut { rect, inner }
    }

    pub fn as_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut{rect:self.rect,inner:self.inner}
    }

    pub fn as_ref(&self)->BBoxRef<N,T>{
        BBoxRef{rect:self.rect,inner:self.inner}
    }
       
}

pub struct BBoxRefPtr<N:NumTrait,T>{
    pub rect:*const axgeom::Rect<N>,
    pub inner:tools::Unique<T>
}





///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user
///cannot modify the bounding box since it is hidden, with only read access.
#[repr(C)]
pub struct BBoxPtr<N: NumTrait, T> {
    rect: axgeom::Rect<N>,
    inner: tools::Unique<T>,
}

impl<N: NumTrait, T> BBoxPtr<N, T> {
    pub fn inner(&self)->&T{
        unsafe{&*self.inner.as_ptr()}
    }
    
    pub fn inner_mut(&mut self)->&mut T{
        unsafe{&mut *self.inner.as_ptr()}
    }

    pub(crate) unsafe fn from_bbox(a:BBox<N,&mut T>)->BBoxPtr<N,T>{
        BBoxPtr{rect:a.rect,inner:tools::Unique::new(a.inner).unwrap()}
    }
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub unsafe fn new(rect: axgeom::Rect<N>, inner: tools::Unique<T>) -> BBoxPtr<N, T> {
        BBoxPtr { rect, inner}
    }

    ///Unsafe since user could call this function
    ///using a mutable reference from inside of a callback function
    ///of a dinotree query function.
    #[inline]
    pub(crate) unsafe fn set_aabb(&mut self,aabb:axgeom::Rect<N>){
        self.rect=aabb;
    }
}

unsafe impl<N: NumTrait, T> HasAabb for BBoxPtr<N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        BBoxRef::new(&self.rect,unsafe{&*self.inner.as_ptr()})
    }
}
unsafe impl<N:NumTrait,T> HasAabbMut for BBoxPtr<N,T>{
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut::new(&self.rect,unsafe{&mut *self.inner.as_ptr()})
    }
}




///A wrapper type where you are allowed to modify the aabb.
#[derive(Copy,Clone)]
#[repr(C)]
pub struct BBoxMut<N:NumTrait,T>{
    pub aabb:axgeom::Rect<N>,
    pub inner:T
}

impl<N:NumTrait,T> BBoxMut<N,T>{
    pub fn new(aabb:axgeom::Rect<N>,inner:T)->BBoxMut<N,T>{
        BBoxMut{aabb,inner}
    }

    pub fn inner_mut(&mut self)->&mut T{
        &mut self.inner
    }
}


pub fn into_bbox_slice<N:NumTrait,T>(arr:&mut [BBoxMut<N,T>])->&mut [BBox<N,T>]{
    unsafe{&mut *(arr as *mut [BBoxMut<_,_>] as *mut [BBox<_,_>])}
}


pub fn into_bbox_mut_slice<N:NumTrait,T>(arr:&mut [BBox<N,T>])->&mut [BBoxMut<N,T>]{
    unsafe{&mut *(arr as *mut [BBox<_,_>] as *mut [BBoxMut<_,_>])}
}







///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user
///cannot modify the bounding box since it is hidden, with only read access.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BBox<N: NumTrait, T> {
    rect: axgeom::Rect<N>,
    pub inner: T,
}

use core::fmt::Debug;
use core::fmt::Formatter;

impl<N: NumTrait + Debug, T: Debug> Debug for BBox<N, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

impl<N: NumTrait, T> BBox<N, T> {
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub unsafe fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }

    ///Unsafe since user could call this function
    ///using a mutable reference from inside of a callback function
    ///of a dinotree query function.
    #[inline]
    pub unsafe fn set_aabb(&mut self,aabb:axgeom::Rect<N>){
        self.rect=aabb;
    }
}

unsafe impl<N: NumTrait, T> HasAabb for BBox<N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        BBoxRef::new(&self.rect,unsafe{&*(&self.inner as *const _)})
    }
}
unsafe impl<N:NumTrait,T> HasAabbMut for BBox<N,T>{
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut::new(&self.rect,unsafe{&mut *(&mut self.inner as *mut _)})
    }
}




