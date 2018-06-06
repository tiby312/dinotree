#![feature(iterator_step_by)]

#![feature(test)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
#[cfg(test)]
extern crate rand;

#[cfg(test)]
extern crate test;

extern crate smallvec;


mod inner_prelude{
  
  //pub use base_kdtree::TreeCache;
  //pub use AABBox;
  pub use axgeom::Axis;
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

///Contains convenience structs.
pub mod support;

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


use axgeom::Rect;
//pub use treetimer::*;

use axgeom::XAXISS;
use axgeom::YAXISS;
//pub use base_kdtree::DivNode;




///The underlying number type used for the dinotree.
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug{}


pub use dyntree::DynTree;
pub use tree_alloc::NodeDyn;
pub use tree_alloc::NdIter;
pub use tree_alloc::NdIterMut;
use dyntree::DynTreeRaw;




pub trait HasAabb{
    type Num:NumTrait;
    fn get(&self)->&axgeom::Rect<Self::Num>;
}





//Pub so benches can access
#[cfg(test)]
mod test_support;

