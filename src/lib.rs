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
    pub(crate) use crate::compt::Depth;
    pub(crate) use crate::compt::Visitor;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::*;
    pub(crate) use crate::elem::*;
    pub(crate) use crate::bbox::*;
    pub(crate) use crate::par;
}


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
    pub use crate::HasAabb;
    pub use crate::HasAabbMut;
    pub use crate::NumTrait;
    pub use crate::par;
}


///Provies some debugging and misc functions.
pub mod tools;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Provies a slice that produces destructured bounding boxes as the elements,
///so as not to give the user mutable references to the elements themselves.
///If the user were to get these, they could swap elements in the tree and violate
///the invariants of the tree.
pub mod elem;

///A collection of different bounding box containers.
pub mod bbox;

///The dinotree data structure.
pub mod dinotree;

///Like a dinotree, but with a more generic interface. This comes at the cost of performance.
///Use this only to compare against the main one.
pub mod dinotree_generic;

///A version of a dinotree where the bots that belong to a node are not
///sorted along an axis. So this is really a regular kd-tree.
pub mod notsorted;


///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait: Ord + Copy + Send + Sync + core::fmt::Debug {}

impl<T> NumTrait for T where T: Ord + Copy + Send + Sync + Unpin + core::fmt::Debug {}




use crate::bbox::BBoxRef;
use crate::bbox::BBoxRefMut;


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
pub unsafe trait HasAabb{
    type Num: NumTrait;
    type Inner;
    fn get(&self) -> BBoxRef<Self::Num,Self::Inner>;
}

///This object can return an aabb and simultaneously return a inner object that can be mutated.
pub unsafe trait HasAabbMut:HasAabb{
    fn get_mut(&mut self)->BBoxRefMut<Self::Num,Self::Inner>;
}






