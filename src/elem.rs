
use crate::inner_prelude::*;

pub struct ElemSliceMut<'a,T:HasAabb>{
    pub(crate) inner:&'a mut ElemSlice<T>
}

impl<'a,T:HasAabbMut> ElemSliceMut<'a,T>{
    pub fn len(&self)->usize{
        self.inner.len()
    }

    pub fn as_mut(&mut self)->ElemSliceMut<T>{
        ElemSliceMut{inner:self.inner}
    }
    pub fn new(inner:&'a mut ElemSlice<T>)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner}
    }
    pub fn split_first_mut(self)->Option<(BBoxRefMut<'a,T::Num,T::Inner>,ElemSliceMut<'a,T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(first,ElemSliceMut{inner}))
    }
    pub fn iter_mut(self)->ElemIterMut<'a,T>{
        self.inner.iter_mut()
    }

    pub fn from_slice_mut(arr:&mut [T])->Self{
        ElemSliceMut{inner:unsafe{&mut *(arr as *mut _ as *mut _)}}
    }

    pub fn truncate_mut(self,start:usize,end:usize)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner:self.inner.truncate_mut(start,end)}
    }
    pub fn truncate_start_mut(self,start:usize)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner:self.inner.truncate_start_mut(start)}
    }
}

impl<'a, T:HasAabbMut> IntoIterator for ElemSliceMut<'a,T> {
    type Item = BBoxRefMut<'a,T::Num,T::Inner>;
    type IntoIter = ElemIterMut<'a, T>;

    fn into_iter(self) -> ElemIterMut<'a, T> {
        let ElemSliceMut{inner}=self;
        inner.iter_mut()
    }
}
impl<'a,T:HasAabb> core::ops::Deref for ElemSliceMut<'a,T> {
    type Target = ElemSlice<T>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}










#[repr(transparent)]
pub struct ElemSlice<T:HasAabb>{
    pub(crate) inner:[T]
}

impl<T:HasAabbMut> ElemSlice<T>{


    pub fn split_first_mut(&mut self)->Option<(BBoxRefMut<T::Num,T::Inner>,&mut ElemSlice<T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(first.get_mut(),Self::from_slice_mut(inner)))
    }
    pub fn iter_mut(&mut self)->ElemIterMut<T>{
        ElemIterMut{ii:self.inner.iter_mut()}
    }

    pub fn from_slice_mut(arr:&mut [T])->&mut Self{
        unsafe{&mut *(arr as *mut _ as *mut _)}
    }

    pub fn truncate_mut(&mut self,start:usize,end:usize)->&mut ElemSlice<T>{
        Self::from_slice_mut(&mut self.inner[start..end])
    }
    pub fn truncate_start_mut(&mut self,start:usize)->&mut ElemSlice<T>{
        Self::from_slice_mut(&mut self.inner[start..])
    }
}

impl<T:HasAabb> ElemSlice<T>{
    
    pub fn from_slice(arr:&[T])->&Self{
        unsafe{& *(arr as *const _ as *const _)}
    }


    pub fn truncate(&self,start:usize,end:usize)->&ElemSlice<T>{
        Self::from_slice(&self.inner[start..end])
    }
    pub fn truncate_start(&self,start:usize)->&ElemSlice<T>{
        Self::from_slice(&self.inner[start..])
    }

    pub fn is_empty(&self)->bool{
        self.inner.is_empty()
    }

    pub fn iter(&self)->ElemIter<T>{
        ElemIter{ii:self.inner.iter()}
    }  

    pub fn len(&self)->usize{
        self.inner.len()
    }  
    
}

pub struct ElemIterMut<'a,T:HasAabbMut>{
    ii:core::slice::IterMut<'a,T>
}

impl<'a,T:HasAabbMut> Iterator for ElemIterMut<'a,T>{
    type Item=BBoxRefMut<'a,T::Num,T::Inner>;
    fn next(&mut self)->Option<Self::Item>{
        self.ii.next().map(|a|a.get_mut())
    }

    fn size_hint(&self)->(usize,Option<usize>){
        self.ii.size_hint()
    }
}


pub struct ElemIter<'a,T:HasAabb>{
    ii:core::slice::Iter<'a,T>
}

impl<'a,T:HasAabb> Iterator for ElemIter<'a,T>{
    type Item=BBoxRef<'a,T::Num,T::Inner>;
    fn next(&mut self)->Option<Self::Item>{
        self.ii.next().map(|a|a.get())
    }

    fn size_hint(&self)->(usize,Option<usize>){
        self.ii.size_hint()
    }
}


impl<'a,T:HasAabb> core::iter::FusedIterator for ElemIter<'a,T>{}
impl<'a,T:HasAabb> core::iter::ExactSizeIterator for ElemIter<'a,T>{}


impl<'a, T:HasAabb> DoubleEndedIterator for ElemIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ii.next_back().map(|a|a.get())
    }


    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
        where Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        self.ii.rfold(init,|acc,a|{
            f(acc,a.get())
        })
    }
}



impl<'a,T:HasAabbMut> core::iter::FusedIterator for ElemIterMut<'a,T>{}
impl<'a,T:HasAabbMut> core::iter::ExactSizeIterator for ElemIterMut<'a,T>{}

impl<'a, T:HasAabbMut> DoubleEndedIterator for ElemIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ii.next_back().map(|a|a.get_mut())
    }


    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
        where Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        self.ii.rfold(init,|acc,a|{
            f(acc,a.get_mut())
        })
    }
}


impl<'a, T:HasAabbMut> IntoIterator for &'a mut ElemSlice<T> {
    type Item = BBoxRefMut<'a,T::Num,T::Inner>;
    type IntoIter = ElemIterMut<'a, T>;

    fn into_iter(self) -> ElemIterMut<'a, T> {
        self.iter_mut()
    }
}

impl<'a, T:HasAabb> IntoIterator for &'a ElemSlice<T> {
    type Item = BBoxRef<'a,T::Num,T::Inner>;
    type IntoIter = ElemIter<'a, T>;

    fn into_iter(self) -> ElemIter<'a, T> {
        self.iter()
    }
}