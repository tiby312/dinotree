use crate::inner_prelude::*;


///Reference to AABB and Reference to Inner.
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



///Reference to AABB and Mutable Reference to Inner.
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


///Pointer to AABB and Mutable Pointer to Inner.
pub(crate) struct BBoxRefPtr<N:NumTrait,T>{
    pub rect:*const axgeom::Rect<N>,
    pub inner:tools::Unique<T>
}


///AABB and mutable pointer to Inner.
#[repr(C)]
pub(crate) struct BBoxPtr<N: NumTrait, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: tools::Unique<T>,
}

impl<N: NumTrait, T> BBoxPtr<N, T> {
    
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub fn new(rect: axgeom::Rect<N>, inner: tools::Unique<T>) -> BBoxPtr<N, T> {
        BBoxPtr { rect, inner}
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


///AABB and mutable pointer to Inner.
#[repr(C)]
pub struct BBoxMut<'a,N: NumTrait, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N: NumTrait, T> BBoxMut<'a,N, T> {
    
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub fn new(rect: axgeom::Rect<N>, inner: &'a mut T) -> BBoxMut<'a,N, T> {
        BBoxMut { rect, inner}
    }

}

unsafe impl<'a,N: NumTrait, T> HasAabb for BBoxMut<'a,N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        BBoxRef::new(&self.rect,self.inner)
    }
}
unsafe impl<'a,N:NumTrait,T> HasAabbMut for BBoxMut<'a,N,T>{
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut::new(&self.rect,self.inner)
    }
}



/*
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
*/


    
pub(crate) struct BBoxSendSync<N:NumTrait,K> {
    pub rect: axgeom::Rect<N>,
    pub inner: K,
}
unsafe impl<N:NumTrait,K> Send for BBoxSendSync<N,K>{}
unsafe impl<N:NumTrait,K> Sync for BBoxSendSync<N,K>{}

unsafe impl<N:NumTrait,K> HasAabb for BBoxSendSync<N,K> {
    type Num = N;
    type Inner = K;
    fn get(&self) -> BBoxRef<Self::Num,Self::Inner> {
        BBoxRef::new(&self.rect,&self.inner)
    }
}
unsafe impl<N:NumTrait,K> HasAabbMut for BBoxSendSync<N,K>{
    fn get_mut(&mut self) -> BBoxRefMut<N,K> {
        BBoxRefMut::new(&self.rect,&mut self.inner)
    }   
}





///AABB and Inner.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BBox<N: NumTrait, T> {
    pub rect: axgeom::Rect<N>,
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
    pub fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
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




