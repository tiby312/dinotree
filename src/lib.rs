#![feature(iterator_step_by)]

#![feature(test)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
//extern crate ordered_float;
#[cfg(test)]
extern crate rand;

#[cfg(test)]
extern crate test;

extern crate smallvec;


mod inner_prelude{
  
  //pub use base_kdtree::TreeCache;
  //pub use AABBox;
  //pub use axgeom::Axis;
  pub use compt::LevelIter;
  pub use compt::Depth;
  pub use axgeom::Range;
  pub use *;
  pub use oned::sweeper_update;

  //pub use super::median::MedianStrat;
  pub use compt::CTreeIterator;
  pub use par;
  pub use axgeom::AxisTrait;
  pub use std::marker::PhantomData;
  pub use treetimer::*;
  pub use NumTrait;
  pub use *;
  pub use tree_alloc::NodeDyn;
}

///Contains the different median finding strategies.
//pub mod median;


///Contains tree level by level timing collection code. 
pub mod treetimer;


pub mod par;

///Contains rebalancing code.
mod base_kdtree;
///Provides low level functionality to construct a dyntree.
mod tree_alloc;


///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory. 
mod dyntree;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///C  ontains misc tools
mod tools;


//use axgeom::Rect;
//pub use treetimer::*;

//use axgeom::XAXISS;
//use axgeom::YAXISS;
//pub use base_kdtree::DivNode;


///Returns the height of what is used internally to construct a dinotree.
pub fn compute_tree_height(num_bots: usize) -> usize {
    
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.
    const NUM_PER_NODE: usize = 12;  

    //8 is worse than 20 which is worse than 12 on android. sticking with 12
    if num_bots <= NUM_PER_NODE {
        return 1;
    } else {
        return (num_bots as f32 / NUM_PER_NODE as f32).log2().ceil() as usize;
    }
}


///The underlying number type used for the dinotree.
pub trait NumTrait:Ord+Copy+Send+Sync{}

impl<T> NumTrait for T
where T: Ord+Copy+Send+Sync
{
}


pub use dyntree::DynTree;
pub use tree_alloc::NodeDyn;
pub use tree_alloc::NdIter;
pub use tree_alloc::NdIterMut;
pub use tree_alloc::LeafDyn;
pub use tree_alloc::NonLeafDyn;
pub use tree_alloc::LeafDynMut;
pub use tree_alloc::NonLeafDynMut;


///Marker trait.
///Elements that are inserted into the tree must have a bounding box.
///Additionally to implemnting get(), implementors must move their 
///bounding boxes while inserted into the tree.
///So the Rect returns by get(), must always be the same once the object is inserted
///into the tree.
///Not doing so would violate invariants of the tree, and would thus make all the 
///query algorithms performed on the tree would not be correct.
///In some cases, violating this rule might even lead to undefined behavior.
///Some algorithms traverse the tree reading the elements aabb, while the user has a mutable reference to an element.
///This case is true for DynTreeExt.
///Its suggested that the user use visilibty to hide the underlying aabb from being modified during
///the query of the tree.
pub trait HasAabb{
    type Num:NumTrait;
    fn get(&self)->&axgeom::Rect<Self::Num>;
}



//Pub so benches can access
#[cfg(test)]
mod test_support;

