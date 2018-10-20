//! Provides the dinotree data structure and ways to traverse it.

#![feature(ptr_internals)]
#![feature(align_offset)]
#![feature(iterator_step_by)]
#![feature(trusted_len)]
#![feature(test)]
extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate is_sorted;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate test;
extern crate smallvec;


mod inner_prelude{
  pub use compt::LevelIter;
  pub use compt::Depth;
  pub use axgeom::Range;
  pub use *;
  pub use oned::sweeper_update;
  pub use compt::CTreeIterator;
  pub use par;
  pub use axgeom::AxisTrait;
  pub use std::marker::PhantomData;
  pub use compt::timer::*;
  pub use NumTrait;
  pub use *;
  pub use tree_alloc::NodeDyn;
}

///Contains code to write generic code that can be run in parallel, or sequentially. Not intended to be used directly by the user.
///Used by algorithms that operate on the tree.
#[doc(hidden)]
pub mod par;

///Contains rebalancing code.
mod base_kdtree;
///Provides low level functionality to construct a dyntree.
mod tree_alloc;

mod assert_invariants;

mod tree_health;
pub use tree_health::LevelRatioIterator;

///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory. 
mod dyntree;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;


pub trait Splitter:Sized{
    fn div(self)->(Self,Self);
    fn add(self,Self)->Self;
    fn node_start(&mut self);
    fn node_end(&mut self);
}

pub struct SplitterEmpty;

impl Splitter for SplitterEmpty{
  fn div(self)->(Self,Self){(SplitterEmpty,SplitterEmpty)}
  fn add(self,_:Self)->Self{SplitterEmpty}
  fn node_start(&mut self){}
  fn node_end(&mut self){}
}

pub fn compute_tree_height_heuristic_debug(num_bots: usize,num_per_node:usize) -> usize {
    
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        return 1;
    } else {
        return (num_bots as f32 / num_per_node as f32).log2().ceil() as usize;
    }
}

///Returns the height of a dyn tree for a given number of bots.
///The height is chosen such that the nodes will each have a small amount of bots.
///If we had a node per bot, the tree would be too big. 
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///This is provided so that users can allocate enough space for all the nodes
///before the tree is constructed, perhaps for some graphics buffer.
pub fn compute_tree_height_heuristic(num_bots: usize) -> usize {
    
    //we want each node to have space for around num_per_node bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.


    //Make this number too small, and the tree will have too many levels,
    //and too much time will be spent recursing.
    //Make this number too high, and you will lose the properties of a tree,
    //and you will end up with just sweep and prune.
    //This number was chosen emprically from running the dinotree_alg_data project,
    //on two different machines.
    //const NUM_PER_NODE: usize = 32;  
    const NUM_PER_NODE: usize = 20;  


    if num_bots <= NUM_PER_NODE {
        return 1;
    } else {
        return (num_bots as f32 / NUM_PER_NODE as f32).log2().ceil() as usize;
    }
}


///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait:Ord+Copy+Send+Sync{}

impl<T> NumTrait for T
where T: Ord+Copy+Send+Sync{}


pub use tree_alloc::FullComp;
pub use dyntree::DynTree;
pub use tree_alloc::NodeDyn;
pub use tree_alloc::NdIter;
pub use tree_alloc::NdIterMut;
pub use dyntree::BBox;


///Marker trait to signify that this object has an axis aligned bounding box.
///Additionally the aabb must not change while the object is contained in the tree.
///Not doing so would violate invariants of the tree, and would thus make all the 
///query algorithms performed on the tree would not be correct.
///
///Not only will the algorithms not be correct, but undefined behavior may be introduced.
///Some algorithms rely on the positions of the bounding boxes to determined if two aabbs can
///be mutably borrowed at the same time. For example the multirect algorithm makes this assumption.
///
///The trait is marked as unsafe. The user is suggested to use the DynTree builder.
///The builder will safely construct a tree of elements wrapped in a Bounding Box where the aabb
///is protected from being modified via visibility. The trait is still useful to keep the querying algorithms generic.
pub unsafe trait HasAabb{
    type Num:NumTrait;
    fn get(&self)->&axgeom::Rect<Self::Num>;
}


///Marker trait to indicate that this object is a point.
pub trait IsPoint{
  type Num:NumTrait;
  fn get_center(&self)->[Self::Num;2];
}

