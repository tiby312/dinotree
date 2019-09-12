use crate::inner_prelude::*;


///Equivalent to: `&mut (Rect<N>,T)`
pub struct BBoxIndirect<'a,N,T>{
    pub inner: &'a mut BBox<N,T>
}


impl<'a,N: NumTrait, T> HasAabb for BBoxIndirect<'a,N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        self.inner.get()
    }
}
impl<'a,N:NumTrait,T> HasInner for BBoxIndirect<'a,N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->BBoxRef<N,T>{
        self.inner.get_inner()
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->BBoxRefMut<N,T>{
        self.inner.get_inner_mut()
    }
}



///Equivalent to: `(&Rect<N>,&T)` 
#[repr(C)]
pub struct BBoxRef<'a,N,T> {
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
pub struct BBoxRefMut<'a,N,T> {
    pub rect: &'a axgeom::Rect<N>,
    pub inner: &'a mut T,
}


impl<'a,N,T> BBoxRefMut<'a,N,T> {
    
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




///Equivalent to: `(Rect<N>,&mut T)` 
///
///If we were to use a `BBox<N,&mut T>`, then `HasAabb::get()` would return a `&mut &mut T`, which is cumbersome.
#[repr(C)]
pub struct BBoxMut<'a,N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N, T> BBoxMut<'a,N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: &'a mut T) -> BBoxMut<'a,N, T> {
        BBoxMut { rect, inner}
    }

}


impl<'a,N: NumTrait, T> HasAabb for BBoxMut<'a,N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<'a,N:NumTrait,T> HasInner for BBoxMut<'a,N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->BBoxRef<N,T>{
        BBoxRef{rect:&self.rect,inner:self.inner}
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut{rect:&self.rect,inner:self.inner}
    }
}





unsafe impl<N:Send,T:Send> Send for BBoxPtr<N,T>{}
unsafe impl<N:Sync,T:Sync> Sync for BBoxPtr<N,T>{}

///Equivalent to: `(Rect<N>,*mut T)` 
#[repr(C)]
pub struct BBoxPtr<N, T> {
    pub rect: axgeom::Rect<N>,
    inner: tools::Unique<T>,
}

impl<'a,N, T> BBoxPtr<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: &mut T) -> BBoxPtr<N, T> {
        BBoxPtr { rect, inner:unsafe{tools::Unique::new_unchecked(inner as *mut _)}}
    }
}


impl<N: NumTrait, T> HasAabb for BBoxPtr<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<N:NumTrait,T> HasInner for BBoxPtr<N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->BBoxRef<N,T>{
        BBoxRef{rect:&self.rect,inner:unsafe{self.inner.as_ref()}}
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut{rect:&self.rect,inner:unsafe{self.inner.as_mut()}}
    }
}






#[derive(Copy, Clone)]
#[repr(C)]
///Equivalent to: `(Rect<N>,T)` 
pub struct BBox<N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: T,
}


impl<N, T> BBox<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}



impl<N: NumTrait, T> HasAabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<N:NumTrait,T> HasInner for BBox<N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->BBoxRef<N,T>{
        BBoxRef{rect:&self.rect,inner:&self.inner}
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->BBoxRefMut<N,T>{
        BBoxRefMut{rect:&self.rect,inner:&mut self.inner}
    }
}

