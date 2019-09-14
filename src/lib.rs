//!
//! 
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
//! to 'cover' all the bots it owns.
//! All the objects in a node are sorted along that node's axis.
//!
//!
//! ~~~~
//! # Overview
//!
//! Provides the dinotree data structure and ways to traverse it. No actual query algorithms are provided in this crate.
//! Only the data structure and a way to construct and traverse it are provided in this crate.
//!
//!
//! ## Data Structure
//!
//! Three flavors of the same fundamental data structure are provided. They each 
//! have different characteristics that may make you want to use them over the others.
//!
//! + `DinoTree` is made up of `(Rect<N>,&mut T)`
//! + `DinoTreeDirect` is made up of `(Rect<N>,T)`
//! + `DinoTreeIndirect` is made up of `&mut (Rect<N>,T)`
//!
//! ## Data Structure Details
//!
//! + `DinoTree` is the most well rounded and most performant in all cases.
//! The aabb's themselves don't have a level of indirection. Broad-phase
//! algorithms need to look at these very often. It's only when these algorithms
//! detect a intersection do they need to look further, which doesnt happen as often.
//! so a level of indirection here is not so bad. The fact that T is a pointer, also
//! means that more aabb's will be in cache at once, further speeding up algorithms
//! that need to look at the aabb's very often.
//!
//! + `DinoTreeDirect` can be fairly fast in cases where there are many, many overlapping
//! elements in the tree, but this comes at the cost of a more expensive base cost
//! of constructing (and deconstructing) the tree. One benefit of using this tree, is
//! that it owns the elements completely, so there are no lifetime references to worry about.
//!
//! + `DinoTreeIndirect` has fast tree construction given that we are just sorting and swapping
//! pointers, but there is no cache-coherence during the query phase, so this can 
//! cause real slow down to query algorithms if there are many overlapping elements.
//!
//! ## BBox Differences
//!
//! `DinoTree` and `DinoTreeDirect` both have the user provide a `&mut [T]` or `Vec<T>` and produce a 
//! `(Rect<N>,&mut T)` or `(Rect<N>,T)` from that slice and a user provided aabb construction function.
//! This was done to minimize total memory used. In most cases, an elements aabb doesnt mean anything
//! unless it exists in a space partitioning tree. So if the tree doesnt exist, 
//! the aabb is just taking up spacing in that object slowing down other algorithms that have to iterating 
//! over all the bots for some other purpose. So this api encourages the user to only make the abb
//! on creation of the tree. 
//!
//! In the case of `DinoTreeIndirect`, we can hardly avoid it, since the tree is made up solely of pointers
//! so the user must provide a slice with the aabb for each object already.
//!
//! ## NotSorted
//!
//! For comparison, a normal kd-tree is provided by `NotSorted`. In this tree, the elements are not sorted
//! along an axis at each level.
//!
//!
//! ## User Protection
//!
//! A lot is done to forbid the user from violating the invariants of the tree once constructed
//! while still allowing them to mutate elements of the tree. The user can mutably traverse down the tree
//! with a `VistrMut` and `ElemSliceMut`, but the elements that are returned have already been destructured in such a way
//! that the user only has read-only access to the `Rect<N>`, even if they do have write access to the inner `T`.
//!
//!
//! ## Usage Guidlines
//!
//! If you insert aabb's with zero width or zero height, it is unspecified behavior.
//! It is expected that all elements in the tree take up some area (just like in real life).
//! 
//!


#![no_std]
#[cfg(all(feature = "unstable", test))]
extern crate test;

extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

mod inner_prelude {
    pub use axgeom::*;
    pub use core::iter::*;
    pub use core::marker::PhantomData;
    

    pub use alloc::vec::Vec;

    pub(crate) use super::*;
    pub(crate) use crate::compt::Depth;
    pub(crate) use crate::compt::Visitor;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::*;
    pub(crate) use crate::elem::*;
    pub(crate) use crate::bbox::*;
    pub(crate) use crate::par;
}


pub use axgeom;
pub use compt;
pub use rayon;

//mod assert_invariants;

///Contains generic code using both all dinotree versions
pub mod tree;


///Contains code to write generic code that can be run in parallel, or sequentially. Not intended to be used directly by the user.
///Used by algorithms that operate on the tree.
pub mod par;

///Prelude to include by using: pub use dinotree::prelude::*
pub mod prelude{
    pub use crate::tree::*;
    pub use crate::elem::*;
    pub use crate::bbox::*;
    
    pub use crate::tree::dinotree_good::*;
    pub use crate::tree::dinotree_indirect::*;
    pub use crate::tree::dinotree_direct::*;
    pub use crate::tree::notsorted::*;
    //pub use crate::dinotree_owned::*;
    //pub use crate::tree::dinotree_direct::*;
    pub use crate::HasAabb;
    pub use crate::HasInner;
    pub use crate::NumTrait;
    pub use crate::par;
}


///Provies some debugging and misc functions.
pub mod tools;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Provies a slice that produces destructured bounding boxes as the elements.
///
/// Return destructured bbox's so as not to give the user mutable references to the elements themselves.
/// If the user were to get these, they could swap elements in the tree and violate
/// the invariants of the tree.
pub mod elem;

///A collection of different bounding box containers.
pub mod bbox;

///The dinotree data structure. The tree is made up of: `(Rect<N>,&mut T)`
//pub mod dinotree;

//pub mod dinotree_owned;

///A version of dinotree where the tree is made up of `(Rect<N>,T)` 
///
///From benchmarks, this is slower than the main dinotree data structure provided.
//pub mod dinotree_direct;


///A version of dinotree where the tree is made up of `&mut (Rect<N>,T)` 
///
///From benchmarks, this is slower than the main dinotree data structure provided.
//pub mod dinotree_indirect;


///A version of a dinotree where the bots are not sorted.
///
///So this is really a regular kd-tree. The bots that belong to a node are not
///sorted along an axis. 
pub mod notsorted;




///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait: Ord + Copy + Send + Sync {}

impl<T> NumTrait for T where T: Ord + Copy + Send + Sync {}




//use crate::bbox::BBoxRef;
//use crate::bbox::BBoxRefMut;
use axgeom::*;

///Marker trait to signify that this object has an axis aligned bounding box.
pub trait HasAabb{
    type Num: NumTrait;
    fn get(&self) -> &Rect<Self::Num>;
}

pub trait HasInner:HasAabb{
    type Inner;
    #[inline(always)]
    fn inner_mut(&mut self)->&mut Self::Inner{
        self.get_inner_mut().1
    }
    #[inline(always)]
    fn inner(&self)->&Self::Inner{
        self.get_inner().1
    }
    fn get_inner(&self)->(&Rect<Self::Num>,&Self::Inner);
    fn get_inner_mut(&mut self)->(&Rect<Self::Num>,&mut Self::Inner);
}

