use crate::inner_prelude::*;


///Equivalent to: `&mut (Rect<N>,T)`
pub struct BBoxIndirect<'a,N:NumTrait,T>{
    pub inner: &'a mut BBox<N,T>
}

impl<'a,N: NumTrait, T> HasAabb for BBoxIndirect<'a,N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        self.inner.get()
    }
}
impl<'a,N:NumTrait,T> HasAabbMut for BBoxIndirect<'a,N,T>{
    #[inline(always)]
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        self.inner.get_mut()
    }
}



///Equivalent to: `(&Rect<N>,&T)` 
#[repr(C)]
pub struct BBoxRef<'a,N:NumTrait,T> {
    pub rect: &'a axgeom::Rect<N>,
    pub inner: &'a T,
}

impl<'a,N:NumTrait,T> BBoxRef<'a, N,T> {  
    #[inline(always)]
    pub fn new(rect: &'a axgeom::Rect<N>, inner: &'a T) -> BBoxRef<'a,N,T> {
        BBoxRef{ rect, inner }
    }
}



///Equivalent to: `(&Rect<N>,&mut T)` 
#[repr(C)]
pub struct BBoxRefMut<'a,N:NumTrait,T> {
    pub rect: &'a axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N:NumTrait,T> BBoxRefMut<'a,N,T> {
    
    #[inline(always)]
    pub fn new(rect: &'a axgeom::Rect<N>, inner: &'a mut T) -> BBoxRefMut<'a,N,T> {
        BBoxRefMut { rect, inner }
    }

    #[inline(always)]
    pub fn as_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut{rect:self.rect,inner:self.inner}
    }

    #[inline(always)]
    pub fn as_ref(&self)->BBoxRef<N,T>{
        BBoxRef{rect:self.rect,inner:self.inner}
    }
       
}


///Pointer to AABB and Mutable Pointer to Inner.
pub(crate) struct BBoxRefPtr<N:NumTrait,T>{
    pub rect:*const axgeom::Rect<N>,
    pub inner:tools::Unique<T>
}



///Equivalent to: `(Rect<N>,&mut T)` 
///
///If we were to use a `BBox<N,&mut T>`, then `HasAabb::get()` would return a `&mut &mut T`, which is cumbersome.
#[repr(C)]
pub struct BBoxMut<'a,N: NumTrait, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N: NumTrait, T> BBoxMut<'a,N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: &'a mut T) -> BBoxMut<'a,N, T> {
        BBoxMut { rect, inner}
    }

}

impl<'a,N: NumTrait, T> HasAabb for BBoxMut<'a,N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        BBoxRef::new(&self.rect,self.inner)
    }
}

impl<'a,N:NumTrait,T> HasAabbMut for BBoxMut<'a,N,T>{
    #[inline(always)]
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut::new(&self.rect,self.inner)
    }
}



#[derive(Copy, Clone)]
#[repr(C)]
///Equivalent to: `(Rect<N>,T)` 
pub struct BBox<N: NumTrait, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: T,
}


impl<N: NumTrait, T> BBox<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}

impl<N: NumTrait, T> HasAabb for BBox<N, T> {
    type Num = N;
    type Inner= T;
    #[inline(always)]
    fn get(&self) -> BBoxRef<N,T>{
        BBoxRef::new(&self.rect,unsafe{&*(&self.inner as *const _)})
    }
}
impl<N:NumTrait,T> HasAabbMut for BBox<N,T>{
    #[inline(always)]
    fn get_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut::new(&self.rect,unsafe{&mut *(&mut self.inner as *mut _)})
    }
}




