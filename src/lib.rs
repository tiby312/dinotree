//!
//! Provides the dinotree data structure and ways to traverse it.
//! All divide and conquer style query algorithms that you can do on this tree would be done using the Vistr nd VistrMut visitors.
//! No actual query algorithms are provided in this crate. Only the data structure and a way to construct it are provided in this crate.
//!
//! # Overview
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
//! Nodes that itersect a divider belong to that node.
//! Every divider keeps track of how thick a line would have to be
//! to cover all the bots it owns.
//!
//! ~~~~
//!
//! # Unsafety
//!
//! The HasAabb trait is marked as unsafe. See its description. 
//!

#![feature(ptr_internals)]
#![feature(test)]


extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate is_sorted;
extern crate itertools;
extern crate reorder;

#[cfg(test)]
extern crate test;

mod inner_prelude{
  pub use tree::*;
  pub use std::mem::*;
  pub use std::marker::PhantomData;
  pub use std::iter::*;
  pub(crate) use compt;
  pub use axgeom::*;
  pub use itertools::Itertools;
  pub use std::time::Instant;
  
  pub(crate) use par;
  pub(crate) use tree;
  pub(crate) use advanced;
  pub(crate) use advanced::Splitter;
  pub(crate) use compt::Depth;
  pub(crate) use compt::Visitor;
  pub(crate) use super::*;
}


///Contains code to write generic code that can be run in parallel, or sequentially. Not intended to be used directly by the user.
///Used by algorithms that operate on the tree.
pub mod par;


///Provides low level functionality to construct a dyntree.

mod assert_invariants;


mod notsorted;

mod tree;

pub use tree::DinoTreeRef;
pub use tree::DinoTreeRefMut;
pub use tree::dinotree::DinoTree;
pub use tree::Vistr;
pub use tree::VistrMut;
pub use tree::FullComp;
pub use tree::NodeRef;
pub use tree::NodeRefMut;
pub use tree::BBox;

///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory. 


mod tools;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains a more complicated api that allows the users to create trees with more control.
///Also provides some debugging functions.
pub mod advanced;



///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug{}

impl<T> NumTrait for T
where T: Ord+Copy+Send+Sync+std::fmt::Debug{}



///Marker trait to signify that this object has an axis aligned bounding box.
///If two HasAabb objects have aabb's that do not intersect, then it must be safe to have a mutable reference
///to each simultaneously. Not upholding this contract can result to undefined behavior so this trait
///is marked unsafe.
///
///Additionally the aabb must not change while the object is contained in the tree.
///Not doing so would violate invariants of the tree, and would thus make all the 
///query algorithms performed on the tree would not be correct.
///
///Not only will the algorithms not be correct, but undefined behavior may be introduced.
///Some algorithms rely on the positions of the bounding boxes to determined if two aabbs can
///be mutably borrowed at the same time. For example the multirect algorithm makes this assumption.
///
///The trait is marked as unsafe. The user is suggested to use the DinoTree builder.
///The builder will safely construct a tree of elements wrapped in a Bounding Box where the aabb
///is protected from being modified via visibility. The trait is still useful to keep the querying algorithms generic.
pub unsafe trait HasAabb{
    type Num:NumTrait;
    fn get(&self)->&axgeom::Rect<Self::Num>;
}



use axgeom::AxisTrait;
pub struct AxisWrap<A:AxisTrait,T>{
    axis:A,
    inner:T
}
impl<A:AxisTrait,T> AxisWrap<A,T>{
    pub fn axis(&self)->A{
        self.axis
    }
    pub fn into_inner(self)->T{
        self.inner
    }
}