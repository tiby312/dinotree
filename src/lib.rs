//!
//! # Overview
//!
//! Provides the dinotree data structure and ways to traverse it. No actual query algorithms are provided in this crate.
//! Only the data structure and a way to construct and traverse it are provided in this crate.
//! 
//! The tree is comprised of copies of objects (rather than references) sorted to improve cache coherency. 
//! There is an alternative NoCopyDinoTree that does not allocate more space,
//! but instead rearranges the bots in a user provided slice for better cache coherency. 
//!
//! ~~~~text
//! 2d Tree Divider Representation:
//!
//!
//!    o   ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┃         ┆         o
//!  ┈┈┈┈┈┈┆     o      o     ┃     o   ┆   o                 o
//!  ───────o─────────────────┃         o┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!                ┆       o  o   o     ┆
//!        o       ┆    o     ┃┈┈┈┈┈o┈┈┈┆       o
//!                ┆   o      ┃         o             o
//!                ┆┈┈┈┈┈┈┈┈┈┈┃         ┆                   o
//!      o         o    o     ┃───────o────────────────────────
//!                ┆          ┃                ┆   o
//!  ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆      o   o   o            ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!     o          ┆          ┃┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆         o
//!          o     ┆   o      ┃        o       ┆   o
//!                ┆          ┃                ┆
//!
//! Axis alternates every level.
//! Divider placement is placed at the median at each level.
//! Objects that intersect a divider belong to that node.
//! Every divider keeps track of how thick a line would have to be
//! to cover all the bots it owns.
//! All the objects in a node are sorted along that node's axis.
//! 
//! ~~~~
//!
//! # Unsafety
//!
//! The HasAabb trait is marked as unsafe. See its description.
//! Unsafety used to have slices of bots in the tree, but also a slice of all the bots
//! so that we can efficiently return a slice of all the bots.
//! Unsafety is used to reuse code between sequential and parallel build algorithms.
//!
//! ## Analysis
//! Please see the [dinotree_report](https://github.com/tiby312/dinotree_report) github project, for a writeup of the design and analysis of the algorithms in this project.
//!


#![no_std]
#[cfg(all(feature = "unstable", test))]
extern crate test;

extern crate alloc;
extern crate axgeom;
extern crate compt;
extern crate is_sorted;
extern crate itertools;
extern crate pdqselect;
extern crate rayon;

mod inner_prelude {
    pub use axgeom::*;
    
    pub use itertools::Itertools;
    pub use core::iter::*;
    pub use core::marker::PhantomData;
    

    pub use alloc::vec::Vec;

    pub(crate) use super::*;
    pub(crate) use crate::advanced::Splitter;
    pub(crate) use crate::compt::Depth;
    pub(crate) use crate::compt::Visitor;
    pub(crate) use crate::advanced::par;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::*;

}


use core::pin::Pin;
pub use assert_invariants::assert_invariants;
mod assert_invariants;

mod tree;

pub use crate::tree::DinoTreeRefTrait;
pub use crate::tree::DinoTreeRefMutTrait;
pub use crate::tree::NodeRef;
pub use crate::tree::NodeRefMut;
pub use crate::tree::Vistr;
pub use crate::tree::VistrMut;


///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory.
mod tools;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;



///A version of dinotree where the elements are copied directly into the tree.
pub mod copy;
///A version where the bots are not copied. This means that the slice borrowed from the user
///must remain borrowed for the entire lifetime of the tree.
pub mod nocopy;
///A version of a dinotree where the bots that belong to a node are not
///sorted along an axis. So this is really a regular kd-tree.
pub mod notsorted;

///Provies some debugging and misc functions.
pub mod advanced;

///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait: Ord + Copy + Send + Sync + core::fmt::Debug {}

impl<T> NumTrait for T where T: Ord + Copy + Send + Sync + Unpin + core::fmt::Debug {}



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
}

pub struct BBoxRefPtr<N:NumTrait,T>{
    pub rect:*const axgeom::Rect<N>,
    pub inner:tools::Unique<T>
}



///Marker trait to signify that this object has an axis aligned bounding box.
///If two HasAabb objects have aabb's that do not intersect, then it must be safe to have a mutable reference
///to each simultaneously.
///
///The aabb must not change while the object is contained in the tree, even though
///many query algorithms will give mutable references to the elements in the tree.
///So multiple calls to get() must return the same bounding box region while the object is in the tree.
///Not doing so would violate invariants of the tree, and would thus make all the
///query algorithms performed on the tree would not be correct.
///
///Not only will the algorithms not be correct, but undefined behavior may be introduced.
///Some algorithms rely on the positions of the bounding boxes to determined if two aabbs can
///be mutably borrowed at the same time. For example the multirect algorithm makes this assumption.
///
///Additionally it should not implement Unpin. Otherwise users could swap elements in the tree.
pub unsafe trait HasAabb{
    type Num: NumTrait;
    type Inner;
    fn get(&self) -> BBoxRef<Self::Num,Self::Inner>;
}
pub unsafe trait HasAabbMut:HasAabb{
    fn get_mut(&mut self)->BBoxRefMut<Self::Num,Self::Inner>;
}








use core::marker::PhantomPinned;

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





pub struct ElemSliceMut<'a,T:HasAabb>{
    inner:&'a mut ElemSlice<T>
}

impl<'a,T:HasAabbMut> ElemSliceMut<'a,T>{

    pub fn as_mut(&mut self)->ElemSliceMut<T>{
        ElemSliceMut{inner:self.inner}
    }
    pub fn new(inner:&'a mut ElemSlice<T>)->ElemSliceMut<'a,T>{
        ElemSliceMut{inner}
    }
    pub fn split_first_mut(self)->Option<(BBoxRefMut<'a,T::Num,T::Inner>,ElemSliceMut<'a,T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(first,ElemSliceMut{inner}))
    }
    pub fn iter_mut(&mut self)->ElemIterMut<T>{
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
    inner:[T]
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
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
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
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
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

/*
pub use crate::pinslice::SlicePin;
mod pinslice{
    use core::pin::Pin;
    use core::slice;

    pub struct SlicePin<T>([T]);
    
    impl<T> SlicePin<T>{
        pub fn from_slice_mut(a:&mut [T])->&mut SlicePin<T>{
            unsafe{&mut *(a as *mut _ as *mut _)}
        }
        pub fn iter_mut(&mut self)->IterMut<T>{
            IterMut(self.0.iter_mut())
        }
        pub fn iter(&self)->slice::Iter<T>{
            self.0.iter()
        }
        pub fn len(&self)->usize{
            self.0.len()
        }
        pub fn truncate(&mut self,start:usize,end:usize)->&mut SlicePin<T>{
            Self::from_slice_mut(&mut self.0[start..end])
        }
        pub fn truncate_start(&mut self,start:usize)->&mut SlicePin<T>{
            Self::from_slice_mut(&mut self.0[start..])
        }
        pub fn split_first_mut(&mut self)->Option<(Pin<&mut T>,&mut SlicePin<T>)>{
            self.0.split_first_mut().map(|(first,rest)|(unsafe{Pin::new_unchecked(first)},Self::from_slice_mut(rest)))
        }
    }


    impl<'a, T> IntoIterator for &'a mut SlicePin<T> {
        type Item = Pin<&'a mut T>;
        type IntoIter = IterMut<'a, T>;

        fn into_iter(self) -> IterMut<'a, T> {
            self.iter_mut()
        }
    }

    impl<'a, T> IntoIterator for &'a SlicePin<T> {
        type Item = &'a T;
        type IntoIter = slice::Iter<'a, T>;

        fn into_iter(self) -> slice::Iter<'a, T> {
            self.iter()
        }
    }


    pub struct IterMut<'a,T>(
        slice::IterMut<'a,T>
    );
    impl<'a,T> Iterator for IterMut<'a,T>{
        type Item=Pin<&'a mut T>;
        fn next(&mut self)->Option<Self::Item>{
            self.0.next().map(|a:&'a mut T|unsafe{Pin::new_unchecked(a)})
        }

        fn size_hint(&self)->(usize,Option<usize>){
            self.0.size_hint()
        }
    }
    
    impl<'a,T> core::iter::FusedIterator for IterMut<'a,T>{}
    impl<'a,T> core::iter::ExactSizeIterator for IterMut<'a,T>{}


    impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.0.next_back().map(|a:&'a mut T|unsafe{Pin::new_unchecked(a)})
        }


        #[inline]
        fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
            where Fold: FnMut(Acc, Self::Item) -> Acc,
        {
            self.0.rfold(init,|acc,a:&'a mut T|{
                f(acc,unsafe{Pin::new_unchecked(a)})
            })
        }
    }


*/