use crate::inner_prelude::*;


///Forbids the user from swapping two nodes around.
pub struct ProtectedNode<'a,T>{
    inner:&'a mut T
}
impl<'a,T:NodeTrait> ProtectedNode<'a,T>{
    pub fn new(inner:&'a mut T)->Self{
        ProtectedNode{inner}
    }

    pub fn get(self)->NodeRef<'a,T::T>{
        self.inner.get()
    }
    pub fn get_mut(self)->NodeRefMut<'a,T::T>{
        self.inner.get_mut()
    }
    pub fn as_ref(&mut self)->ProtectedNode<T>{
        ProtectedNode{inner:self.inner}
    }
}
/*
impl<'a,T:NodeTrait> NodeTrait for ProtectedNode<'a,T>{
    type T=T::T;
    type Num=T::Num;
    fn get(&self)->NodeRef<Self::T>{
        self.inner.get()
        //NodeRef{bots:self.inner.range.as_ref(),cont:&self.inner.cont,div:&self.inner.div}
    }
    fn get_mut(&mut self)->NodeRefMut<Self::T>{
        //NodeRefMut{bots:ProtectedBBoxSlice::new(self.inner.range.as_mut()),cont:&self.inner.cont,div:&self.inner.div}
        self.inner.get_mut()
    }

}
*/

///Forbids the user from swapping aabb's around.
#[repr(transparent)]
pub struct ProtectedBBox<'a,T>{
    inner:&'a mut T
}


impl<'a,T> ProtectedBBox<'a,T>{
    #[inline(always)]
    pub fn as_mut(&mut self)->ProtectedBBox<T>{
        ProtectedBBox{inner:self.inner}
    }

}
impl<'a,T:HasAabb> ProtectedBBox<'a,T>{
    ///TODO talk to this
    #[inline(always)]
    pub fn get_aabb_and_self(&mut self)->(&Rect<T::Num>,&mut ProtectedBBox<'a,T>){
        let rect=unsafe{&*(self.get() as *const _)};
        (rect,self)
    }
}


impl<'a,T:HasAabb> HasAabb for ProtectedBBox<'a,T>{
    type Num=T::Num;
    #[inline(always)]
    fn get(&self)->&Rect<Self::Num>{
        self.inner.get()
    }
}
impl<'a,T:HasInner> HasInner for ProtectedBBox<'a,T>{
    type Inner=T::Inner;
    #[inline(always)]
    fn get_inner(&self)->(&Rect<T::Num>,&Self::Inner){
        self.inner.get_inner()
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<T::Num>,&mut Self::Inner){
        self.inner.get_inner_mut()
    }
}




impl<'a,T> core::borrow::Borrow<T> for ProtectedBBox<'a,T>{
    #[inline(always)]
    fn borrow(&self)->&T{
        self.inner
    }
}

impl<'a,T> AsRef<T> for ProtectedBBox<'a,T>{
    #[inline(always)]
    fn as_ref(&self)->&T{
        self.inner
    }
}









impl<'a,T> core::borrow::Borrow<[T]> for ProtectedBBoxSlice<'a,T>{
    #[inline(always)]
    fn borrow(&self)->&[T]{
        self.inner
    }
}




impl<'a, T> core::iter::IntoIterator for ProtectedBBoxSlice<'a,T> {
    type Item = ProtectedBBox<'a,T>;
    type IntoIter = ProtectedBBoxIter<'a,T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a,T> AsRef<[T]> for ProtectedBBoxSlice<'a,T>{
    #[inline(always)]
    fn as_ref(&self)->&[T]{
        self.inner
    }
}


///Forbids the user from swapping mutable slices in the nodes around.
pub struct ProtectedBBoxSlice<'a,T>{
    inner:&'a mut [T]
}

impl<'a,T> ProtectedBBoxSlice<'a,T>{
    
    #[inline(always)]
    pub fn len(&self)->usize{
        self.inner.len()
    }

    #[inline(always)]
    pub fn split_first_mut(self)->Option<(ProtectedBBox<'a,T>,ProtectedBBoxSlice<'a,T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(ProtectedBBox{inner:first},ProtectedBBoxSlice::new(inner)))
    }


    #[inline(always)]
    pub fn truncate_to(self,a:core::ops::RangeTo<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]}
    }
    #[inline(always)]
    pub fn truncate_from(self,a:core::ops::RangeFrom<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]} 
    }


    #[inline(always)]
    pub fn truncate(self,a:core::ops::Range<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]}
    }

    #[inline(always)]
    pub fn as_mut(&mut self)->ProtectedBBoxSlice<T>{
        ProtectedBBoxSlice{inner:self.inner}
    }

    #[inline(always)]
    pub fn new(inner:&'a mut [T])->Self{
        ProtectedBBoxSlice{inner}
    }

    #[inline(always)]
    pub fn iter(self)->core::slice::Iter<'a,T>{
        self.inner.iter()
    }
    #[inline(always)]
    pub fn iter_mut(self)->ProtectedBBoxIter<'a,T>{
        ProtectedBBoxIter{inner:self.inner.iter_mut()}
    }
}

pub struct ProtectedBBoxIter<'a,T>{
    inner:core::slice::IterMut<'a,T>
}
impl<'a,T> Iterator for ProtectedBBoxIter<'a,T>{
    type Item=ProtectedBBox<'a,T>;
    fn next(&mut self)->Option<ProtectedBBox<'a,T>>{
        self.inner.next().map(|inner|ProtectedBBox{inner})
    }
}









/*
pub struct ElemSliceMut<'a,T:HasAabb>{
    pub(crate) inner:&'a mut ElemSlice<T>
}

impl<'a,T:HasAabbMut> ElemSliceMut<'a,T>{
    #[inline(always)]
    pub fn len(&self)->usize{
        self.inner.len()
    }

    #[inline(always)]
    pub fn as_mut(&mut self)->ElemSliceMut<T>{
        ElemSliceMut{inner:self.inner}
    }

    #[inline(always)]
    pub fn new(inner:&'a mut ElemSlice<T>)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner}
    }

    #[inline(always)]
    pub fn split_first_mut(self)->Option<(BBoxRefMut<'a,T::Num,T::Inner>,ElemSliceMut<'a,T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(first,ElemSliceMut{inner}))
    }

    #[inline(always)]
    pub fn iter_mut(self)->ElemIterMut<'a,T>{
        self.inner.iter_mut()
    }
    #[inline(always)]
    pub fn from_slice_mut(arr:&mut [T])->Self{
        ElemSliceMut{inner:unsafe{&mut *(arr as *mut _ as *mut _)}}
    }
    #[inline(always)]
    pub fn truncate_mut(self,start:usize,end:usize)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner:self.inner.truncate_mut(start,end)}
    }
    #[inline(always)]
    pub fn truncate_start_mut(self,start:usize)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner:self.inner.truncate_start_mut(start)}
    }
}

impl<'a, T:HasAabbMut> IntoIterator for ElemSliceMut<'a,T> {
    type Item = BBoxRefMut<'a,T::Num,T::Inner>;
    type IntoIter = ElemIterMut<'a, T>;
    #[inline(always)]
    fn into_iter(self) -> ElemIterMut<'a, T> {
        let ElemSliceMut{inner}=self;
        inner.iter_mut()
    }
}
impl<'a,T:HasAabb> core::ops::Deref for ElemSliceMut<'a,T> {
    type Target = ElemSlice<T>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}










#[repr(transparent)]
pub struct ElemSlice<T:HasAabb>{
    pub(crate) inner:[T]
}

impl<T:HasAabbMut> ElemSlice<T>{

    #[inline(always)]
    pub fn iter_mut(&mut self)->ElemIterMut<T>{
        ElemIterMut{ii:self.inner.iter_mut()}
    }

    #[inline(always)]
    pub fn split_first_mut(&mut self)->Option<(BBoxRefMut<T::Num,T::Inner>,&mut ElemSlice<T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(first.get_mut(),Self::from_slice_mut(inner)))
    }
}

impl<T:HasAabb> ElemSlice<T>{



    #[inline(always)]
    pub fn from_slice_mut(arr:&mut [T])->&mut Self{
        unsafe{&mut *(arr as *mut _ as *mut _)}
    }

    #[inline(always)]
    pub fn truncate_mut(&mut self,start:usize,end:usize)->&mut ElemSlice<T>{
        Self::from_slice_mut(&mut self.inner[start..end])
    }

    #[inline(always)]
    pub fn truncate_start_mut(&mut self,start:usize)->&mut ElemSlice<T>{
        Self::from_slice_mut(&mut self.inner[start..])
    }
}

impl<T:HasAabb> ElemSlice<T>{
    

    #[inline(always)]
    pub fn from_slice(arr:&[T])->&Self{
        unsafe{& *(arr as *const _ as *const _)}
    }

    #[inline(always)]
    pub fn truncate(&self,start:usize,end:usize)->&ElemSlice<T>{
        Self::from_slice(&self.inner[start..end])
    }
    #[inline(always)]
    pub fn truncate_start(&self,start:usize)->&ElemSlice<T>{
        Self::from_slice(&self.inner[start..])
    }

    #[inline(always)]
    pub fn is_empty(&self)->bool{
        self.inner.is_empty()
    }

    #[inline(always)]
    pub fn iter(&self)->ElemIter<T>{
        ElemIter{ii:self.inner.iter()}
    }  

    #[inline(always)]
    pub fn len(&self)->usize{
        self.inner.len()
    }  
    
}

pub struct ElemIterMut<'a,T:HasAabbMut>{
    ii:core::slice::IterMut<'a,T>
}

impl<'a,T:HasAabbMut> Iterator for ElemIterMut<'a,T>{
    type Item=BBoxRefMut<'a,T::Num,T::Inner>;
    #[inline(always)]
    fn next(&mut self)->Option<Self::Item>{
        self.ii.next().map(|a|a.get_mut())
    }
    #[inline(always)]
    fn size_hint(&self)->(usize,Option<usize>){
        self.ii.size_hint()
    }
}


pub struct ElemIter<'a,T:HasAabb>{
    ii:core::slice::Iter<'a,T>
}

impl<'a,T:HasAabb> Iterator for ElemIter<'a,T>{
    type Item=BBoxRef<'a,T::Num,T::Inner>;
    #[inline(always)]
    fn next(&mut self)->Option<Self::Item>{
        self.ii.next().map(|a|a.get())
    }
    #[inline(always)]
    fn size_hint(&self)->(usize,Option<usize>){
        self.ii.size_hint()
    }
}


impl<'a,T:HasAabb> core::iter::FusedIterator for ElemIter<'a,T>{}
impl<'a,T:HasAabb> core::iter::ExactSizeIterator for ElemIter<'a,T>{}


impl<'a, T:HasAabb> DoubleEndedIterator for ElemIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ii.next_back().map(|a|a.get())
    }


    #[inline(always)]
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
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ii.next_back().map(|a|a.get_mut())
    }


    #[inline(always)]
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

    #[inline(always)]
    fn into_iter(self) -> ElemIterMut<'a, T> {
        self.iter_mut()
    }
}

impl<'a, T:HasAabb> IntoIterator for &'a ElemSlice<T> {
    type Item = BBoxRef<'a,T::Num,T::Inner>;
    type IntoIter = ElemIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> ElemIter<'a, T> {
        self.iter()
    }
}


*/