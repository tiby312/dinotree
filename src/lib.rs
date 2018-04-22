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
  pub use AABBox;
  pub use axgeom::Axis;
  pub use compt::LevelIter;
  pub use compt::Depth;
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


pub mod prelude{
  //pub use base_kdtree::TreeCache;
  pub use tree_alloc::NodeDyn;
  pub use tree_alloc::NdIter;
  pub use treetimer::*;
  pub use daxis;
  pub use AABBox;
  //pub use DepthLevel;
  pub use NumTrait;
  pub use SweepTrait;
  pub use oned::sweeper_update;
  pub use DynTreeRaw;
  //pub use median::*;
  //pub use median::relax::*;
  //pub use median::strict::*;
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
pub mod base_kdtree;
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

use axgeom::Rect;
pub use treetimer::*;

use axgeom::XAXISS;
use axgeom::YAXISS;
pub use base_kdtree::DivNode;

/*
//Note this doesnt check all invariants.
//e.g. doesnt check that every bot is in the tree only once.
fn assert_invariant<T:SweepTrait>(d:&DinoTree2<T>){
    
    let level=d.0.get_level_desc();
    let ll=compt::LevelIter::new(d.0.get_iter(),level);
    use compt::CTreeIterator;
    for (level,node) in ll.dfs_preorder_iter(){
       
       //println!("level={:?}",level.get_depth());
       if level.get_depth()%2==0{
          oned::is_sorted::<A::Next,_>(&node.range);


          let kk=node.container_box;
          for a in node.range.iter(){
             let (p1,p2)=(
                  a.get().0.get().get_range2::<A>().left(),
                  a.get().0.get().get_range2::<A>().right());
              assert!(kk.left()<=p1);
              assert!(p2<=kk.right());
          }
       }else{
          oned::is_sorted::<A,_>(&node.range);
          
          let kk=node.container_box;
          for a in node.range.iter(){
             let (p1,p2)=(
                  a.get().0.get().get_range2::<A::Next>().left(),
                  a.get().0.get().get_range2::<A::Next>().right());
              assert!(kk.left()<=p1);
              assert!(p2<=kk.right());
          }
       }
    }       
    
}
*/


///Returns the level at which a parallel divide and conqur algorithm will switch to sequential



///The underlying number type used for the bounding boxes,
///and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug{}


///A bounding box made up of x and y ordered pairs.
///The left must be less than or equal the right.
#[derive(Copy,Clone)]
pub struct AABBox<N:NumTrait>(pub axgeom::Rect<N>);  //TODO make private 
impl<N:NumTrait> AABBox<N>{

  ///For both axises, the first value must be less than or equal the second.
  ///Panics if not the case.
  //TODO make pass arrays!
  pub fn new(xaxis:(N,N),yaxis:(N,N))->AABBox<N>{
    assert!(xaxis.0<=xaxis.1);
    assert!(yaxis.0<=yaxis.1);
    AABBox(axgeom::Rect::new(xaxis.0,xaxis.1,yaxis.0,yaxis.1))
  }
  
  ///For both axises, the first value must be less than or equal the second.
  ///Panics if not the case.
  pub fn from_array(arr:[N;4])->AABBox<N>{
    assert!(arr[0]<=arr[1]);
    assert!(arr[2]<=arr[3]);
    AABBox(axgeom::Rect::new(arr[0],arr[1],arr[2],arr[3]))
  }
  pub fn get(&self)->((N,N),(N,N)){
    let a=self.0.get_range2::<XAXISS>();
    let b=self.0.get_range2::<YAXISS>();
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
pub use dyntree::DynTreeRaw;
pub use median::MedianStrat;



pub mod par{
    use rayon;
    use compt::Depth;

    pub trait Joiner:Send+Sync+Copy+Clone{
        fn new(d:Depth)->Self;
        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB);
        //fn is_parallel(&self)->bool;
        fn into_seq(&self)->Sequential;
        fn should_switch_to_sequential(&self,a:Depth)->bool;
    }

    #[derive(Copy,Clone)]
    pub struct Parallel(Depth);
    impl Joiner for Parallel{
        fn new(d:Depth)->Self{
          Parallel(d)
        }

        fn into_seq(&self)->Sequential{
          Sequential
        }

        fn should_switch_to_sequential(&self,a:Depth)->bool{
          //Seems like 6 is ideal for my dell xps laptop
          //8 is best on my android phone.
          a.0>=(self.0).0
        }

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
          rayon::join(oper_a, oper_b)
        }
    }

    #[derive(Copy,Clone)]
    pub struct Sequential;
    impl Joiner for Sequential{
        fn new(_:Depth)->Self{
          Sequential
        }
        fn into_seq(&self)->Sequential{
          Sequential
        }

        fn should_switch_to_sequential(&self,_a:Depth)->bool{
           true
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


/*
fn assert_correctness(&self,tree:&KdTree,botman:&BotMan)->bool{
    for (level,axis) in kd_axis::AxisIter::with_axis(tree.tree.get_level_iter()) {
        if level.get_depth()!=tree.tree.get_height()-1{
            for n in level.iter(){
                let no=tree.tree.get_node(n);
                let cont_box=&no.container_box;// no.get_divider_box(&botman.prop,axis);

                let arr=&tree.collision_botids[no.container.get_range().as_int_range()];
                for b in arr{
                    let bot=botman.cont.get_bot(*b);
                    let circle=&botman.as_circle(bot);
                    assert!(cont_box.contains_circle(circle),"{:?}\n{:?}\n{:?}\n{:?}",no,(level,axis),cont_box,circle);
                }
            }
        }
        
    }
     

    let arr=&tree.collision_botids[tree.no_fit.end.0..];
    let mut cols=0;
    for (i, el1) in arr.iter().enumerate() {
        for el2 in arr[i + 1..].iter() {
            let bb=(*el1,*el2);
            let bots = botman.cont.get_bbotpair(bb);

            match bot::is_colliding(&botman.prop, bots) {
                Some(_) => {
                    cols+=1;
                }
                None => {
                }
            }
        }
    }

    let mut cls=0;
    for k in self.binner_helps.iter(){
        cls+=k.cols_found.len();
    }

    let lookup=|a:(BotIndex, BotIndex)|{
        for k in self.binner_helps.iter(){
            for j in k.cols_found.iter(){
                let aa=( (j.inds.0).0 ,(j.inds.1).0);
                let bb=((a.0).0,(a.1).0);
                if aa.0==bb.0 &&aa.1==bb.1{
                    return true;
                }
                if aa.0==bb.1 && aa.1==bb.0{
                    return true;
                }
            }
        }
        false            
    };
    if cols!=cls{
        println!("Cols fail! num collision exp:{:?},  calculated:{:?}",cols,cls);

        for (i, el1) in arr.iter().enumerate() {
            for el2 in arr[i + 1..].iter() {
                let bb=(*el1,*el2);
                let bots = botman.cont.get_bbotpair(bb);

                match bot::is_colliding(&botman.prop, bots) {
                    Some(_) => {
                        if !lookup(bb){
                            println!("Couldnt find {:?}",(bb,bots));

                            println!("in node:{:?}",(lookup_in_tree(tree,bb.0),lookup_in_tree(tree,bb.1)));
                            let n1=lookup_in_tree(tree,bb.0).unwrap();
                            let n2=lookup_in_tree(tree,bb.1).unwrap();
                            let no1=tree.tree.get_node(n1);
                            let no2=tree.tree.get_node(n2);
                            
                            println!("INTERSECTS={:?}",no1.cont.border.intersects_rect(&no2.cont.border));

                        }
                    }
                    None => {
                    }
                }
            }
        }
        assert!(false);
    }
    
    fn lookup_in_tree(tree:&BaseTree,b:BotIndex)->Option<NodeIndex>{
        for level in tree.tree.get_level_iter(){
            for nodeid in level.iter().rev() {
                
                let n = tree.tree.get_node(nodeid);
            
                let k=n.container.get_range().as_int_range();

                let arr=&tree.collision_botids[k];
                for i in arr{
                    if b.0==i.0{
                        return Some(nodeid);
                    }
                }
            }
        }
        return None
    }
    true
}*/