//!
//! # Overview
//!
//! Provides the dinotree data structure and ways to traverse it. No actual query algorithms are provided in this crate.
//! Only the data structure and a way to construct and traverse it are provided in this crate.
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
//! to cover all the bots it owns.
//! All the objects in a node are sorted along that node's axis.
//!
//!
//!
//! What happens if you insert aabbs of zero size.
//! 
//! ~~~~
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

mod assert_invariants;

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
    pub use crate::dinotree::*;
    pub use crate::dinotree_direct::*;
    pub use crate::dinotree_indirect::*;
    pub use crate::HasAabb;
    pub use crate::HasAabbMut;
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
pub mod dinotree;


///A version of dinotree where the tree is made up of `(Rect<N>,T)` 
///
///From benchmarks, this is slower than the main dinotree data structure provided.
pub mod dinotree_direct;


///A version of dinotree where the tree is made up of `&mut (Rect<N>,T)` 
///
///From benchmarks, this is slower than the main dinotree data structure provided.
pub mod dinotree_indirect;


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




use crate::bbox::BBoxRef;
use crate::bbox::BBoxRefMut;


///Marker trait to signify that this object has an axis aligned bounding box.
pub trait HasAabb{
    type Num: NumTrait;
    type Inner;
    fn get(&self) -> BBoxRef<Self::Num,Self::Inner>;
}

///This object can return an aabb and simultaneously return a inner object that can be mutated.
pub trait HasAabbMut:HasAabb{
    fn get_mut(&mut self)->BBoxRefMut<Self::Num,Self::Inner>;
}






