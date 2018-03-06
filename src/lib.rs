#![feature(iterator_step_by)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
#[cfg(test)]
extern crate rand;
extern crate smallvec;


mod inner_prelude{
  
  //pub use base_kdtree::TreeCache;
  pub use AABBox;
  pub use axgeom::Axis;
  pub use compt::LevelIter;
  pub use compt::LevelDesc;
  pub use axgeom::Range;
  pub use *;
  pub use oned::sweeper_update;

  pub use super::median::MedianStrat;
  pub use compt::CTreeIterator;
  pub use par;
  pub use axgeom::AxisTrait;
  pub use std::marker::PhantomData;
  pub use treetimer::*;
  pub use NumTrait;
  pub use *;
  pub use tree_alloc::NodeDyn;
}

/// Conveniently include commonly used symbols in this crate.
/// Use like this:
/// ```
/// extern crate dinotree;
/// use dinotree::prelude::*;
/// fn main(){
///    //...
/// }
/// ```
pub mod prelude{
  //pub use base_kdtree::TreeCache;
  pub use tree_alloc::NodeDyn;
  pub use dyntree::NdIter;
  pub use treetimer::*;
  pub use daxis;
  pub use AABBox;
  pub use DepthLevel;
  pub use NumTrait;
  pub use SweepTrait;
  pub use oned::sweeper_update;
  pub use median::*;
  pub use median::relax::*;
  pub use median::strict::*;
  pub use par;
  pub use treetimer;
  pub use support::*;
}


///Contains the different median finding strategies.
pub mod median;

///Contains convenience structs.
pub mod support;

///Contains tree level by level timing collection code. 
pub mod treetimer;

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


//pub use base_kdtree::TreeCache;
use compt::LevelDesc;
use axgeom::Rect;
use treetimer::*;
use axgeom::XAXIS_S;
use axgeom::YAXIS_S;
pub use base_kdtree::DivNode;


///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    fn switch_to_sequential(a:LevelDesc)->bool;
}

///The underlying number type used for the bounding boxes,
///and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}


#[derive(Copy,Clone)]
pub struct AABBox<N:NumTrait>(pub axgeom::Rect<N>);
impl<N:NumTrait> AABBox<N>{

  ///For both axises, the first value must be less than the second.
  pub fn new(xaxis:(N,N),yaxis:(N,N))->AABBox<N>{
    AABBox(axgeom::Rect::new(xaxis.0,xaxis.1,yaxis.0,yaxis.1))
  }
  pub fn get(&self)->((N,N),(N,N)){
    let a=self.0.get_range2::<XAXIS_S>();
    let b=self.0.get_range2::<YAXIS_S>();
    ((a.start,a.end),(b.start,b.end))
  }
}


pub mod daxis{
  pub use axgeom::Axis as DAxis;
  pub use axgeom::XAXIS;
  pub use axgeom::YAXIS;
}

impl<N:NumTrait> std::fmt::Debug for AABBox<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (xx,yy)=self.get();
        write!(f, "AABBox {{ xaxis: {:?}, yaxis: {:?} }}", xx, yy)
    }
}

///The interface through which the tree interacts with the objects being inserted into it.
pub trait SweepTrait:Send+Sync{
    ///The part of the object that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree. (it might need to be moved
    ///to a different node)
    type Inner:Send+Sync;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;


    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a AABBox<Self::Num>,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a AABBox<Self::Num>,&'a Self::Inner);
}

pub use dyntree::DynTree;
pub use median::MedianStrat;
use support::DefaultDepthLevel;



pub mod par{
    use rayon;
    pub trait Joiner{

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB);
        fn is_parallel()->bool;
    }

    pub struct Parallel;
    impl Joiner for Parallel{
        fn is_parallel()->bool{
            return true;
        }

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
          rayon::join(oper_a, oper_b)
        }
    }
    pub struct Sequential;
    impl Joiner for Sequential{
        fn is_parallel()->bool{
            return false;
        }
        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
            let a = oper_a();
            let b = oper_b();
            (a, b)
        }
    }
}





//Pub so benches can access
#[cfg(test)]
mod test_support;

#[cfg(test)]
mod dinotree_test;
