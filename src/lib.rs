//!
//! Provides the dinotree data structure and ways to traverse it.
//! All divide and conquer style query algorithms that you can do on this tree would be done using the Vistr nd VistrMut visitors.
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
//! Compact Data layout:
//!
//!              xo.....
//!     xo...    |           xo.....
//!  x..|    x...|      x....|      x....
//!  |  |    |   |      |    |      | 
//!  ------------------------------------
//!
//! where:
//! x=data every node has (e.g. number of aabb objects).
//! o=data only non leaf nodes have (e.g. divider location).
//! .=a aabb object. Notice nodes can each have a different number of aabb objects.
//! 
//! Every 'o' has a pointer to the left and right children 'x' s.
//! ~~~~
//!
//! # Unsafety
//!
//! Unsafety is used to construct the tree in contiguous memory despite nodes having each having different sizes.
//!
//! The HasAabb trait is marked as unsafe. See its description. 
//!
//! Parts of the tree may be unsafely uninitialized, but their access is inhibited from the safe Vistr and VistrMut api. The Vistr and VistrMut Visitors will
//! return an Option for their NonLeafNode item. A value of none indicates that that node and all nodes beneath it do
//! not have any aabb objects. In such cases, there is no median from which to create a divider.
//! Internally, these nodes are not initialized since the construction algorithm stops recursing once
//! there are no more aabb objects to be added to a node.
//!

#![feature(specialization)]
#![feature(ptr_internals)]
#![feature(align_offset)]
#![feature(trusted_len)]
#![feature(test)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate is_sorted;

#[cfg(test)]
extern crate test;

mod inner_prelude{
  pub use compt::*;
  pub use axgeom::*;
  pub use *;
}

///Contains code to write generic code that can be run in parallel, or sequentially. Not intended to be used directly by the user.
///Used by algorithms that operate on the tree.
pub mod par;

///Contains rebalancing code.
mod dinotree_inner;
///Provides low level functionality to construct a dyntree.
mod tree_alloc;

mod assert_invariants;

mod tree_health;

///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory. 
mod dinotree;
pub use dinotree::iter_const::TreeIter;
pub use dinotree::iter_mut::TreeIterMut;



///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains a more complicated api that allows the users to create trees with more control.
///Also provides some debugging functions.
pub mod advanced;



///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait:Ord+Copy+Send+Sync{}

impl<T> NumTrait for T
where T: Ord+Copy+Send+Sync{}


pub use tree_alloc::FullComp;
pub use dinotree::DinoTree;
pub use tree_alloc::NodeDyn;
pub use tree_alloc::Vistr;
pub use tree_alloc::VistrMut;
pub use dinotree::BBox;


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
