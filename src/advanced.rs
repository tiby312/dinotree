
use inner_prelude::*;
use axgeom::Rect;

use std::time::Instant;
use dinotree::new_inner;

use dinotree_inner::DefaultSorter;
use dinotree_inner::NoSorter;


fn into_secs(elapsed:std::time::Duration)->f64{
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    sec
}

///Measure the time each level of a recursive algorithm takes that supports the Splitter trait.
///Note that the number of elements in the returned Vec could be less than the height of the tree.
///This can happen if the recursive algorithm does not recurse all the way to the leafs because it
///deemed it not necessary.
pub struct LevelTimer{
    levels:Vec<f64>,
    time:Option<Instant>,
}

impl LevelTimer{
    #[inline]
    pub fn new()->LevelTimer{
        LevelTimer{levels:Vec::new(),time:None}
    }

    #[inline]
    pub fn with_height(height:usize)->LevelTimer{
        LevelTimer{levels:Vec::with_capacity(height),time:None}
    }
    #[inline]
    pub fn into_inner(self)->Vec<f64>{
        self.levels
    }
    #[inline]
    fn node_end_common(&mut self){

        let time=self.time.unwrap();

        let elapsed=time.elapsed();
        self.levels.push(into_secs(elapsed));
        self.time=None;
    }
}
impl Splitter for LevelTimer{
    #[inline]
    fn div(&mut self)->Self{
        self.node_end_common();

        let length=self.levels.len();

        LevelTimer{levels:std::iter::repeat(0.0).take(length).collect(),time:None}
    }
    #[inline]
    fn add(&mut self,a:Self){
        let len=self.levels.len();
        for (a,b) in self.levels.iter_mut().zip(a.levels.iter()){
            *a+=*b;
        }
        if len<a.levels.len(){
            self.levels.extend_from_slice(&a.levels[len..]);
        }
        /*
        let (smaller,mut larger)=if self.levels.len()<a.levels.len(){
            (*self,a)
        }else{
            (a,*self)
        };


        for (a,b) in larger.levels.iter_mut().zip(smaller.levels.iter()){
            *a+=*b;
        }
        *self=larger;
        */
    }
    #[inline]
    fn node_start(&mut self){
        assert!(self.time.is_none());
        self.time=Some(Instant::now());
    }
    #[inline]
    fn node_end(&mut self){
        self.node_end_common();
    } 
}


///Outputs the height given an desirned number of bots per node.
#[inline]
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

#[inline]
pub fn compute_default_level_switch_sequential()->usize{
    const DEPTH_SEQ:usize=2;

    DEPTH_SEQ
}

///Returns the height of a dyn tree for a given number of bots.
///The height is chosen such that the nodes will each have a small amount of bots.
///If we had a node per bot, the tree would be too big. 
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///This is provided so that users can allocate enough space for all the nodes
///before the tree is constructed, perhaps for some graphics buffer.
#[inline]
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



///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter:Sized{

    ///Called to split this into two to be passed to the children.
    fn div(&mut self)->Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self,Self);

    ///Called at the start of the recursive call.
    fn node_start(&mut self);

    ///It is important to note that this gets called in other places besides in the final recursive call of a leaf.
    ///They may get called in a non leaf if its found that there is no more need to recurse further.
    fn node_end(&mut self);
}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty{
  fn div(&mut self)->Self{SplitterEmpty}
  fn add(&mut self,_:Self){}
  fn node_start(&mut self){}
  fn node_end(&mut self){}
}




//Todo use this
pub struct NotSorted<A:AxisTrait,N,T:HasAabb>(pub DinoTree<A,N,T>);

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> NotSorted<A,N,BBox<Num,T>>{
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->NotSorted<A,N,BBox<Num,T>>{
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;

        //See the data project for reasoning behind this value.
        const DEPTH_SEQ:usize=2;

        let gg=if height<=DEPTH_SEQ{
            0
        }else{
            height-DEPTH_SEQ
        };
        
        let dlevel=par::Parallel::new(Depth(gg));

        NotSorted(new_inner(axis,n,bots,aabb_create,&mut ka,height,dlevel,NoSorter))
    }
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->NotSorted<A,N,BBox<Num,T>>{
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;

        let dlevel=par::Sequential;//Parallel::new(Depth(gg));

        NotSorted(new_inner(axis,n,bots,aabb_create,&mut ka,height,dlevel,NoSorter))
    }
}


///A more advanced tree construction function where the use can choose, the height of the tree, the height at which to switch to sequential recursion, and a splitter callback (useful to measuring the time each level of the tree took, for example).
#[inline]
pub fn new_adv<A:AxisTrait,N:Copy,Num:NumTrait,T:Copy,K:Splitter+Send>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:usize,splitter:&mut K,height_switch_seq:usize)->DinoTree<A,N,BBox<Num,T>>{   
    
    let gg=if height<=height_switch_seq{
        0
    }else{
        height-height_switch_seq
    };
    
    let dlevel=par::Parallel::new(Depth(gg));

    new_inner(axis,n,bots,aabb_create,splitter,height,dlevel,DefaultSorter)    
}

///Provides many of the same arguments as new_adv, with the exception of the height at which to switch to sequential, since this is already sequential.
#[inline]
pub fn new_adv_seq<A:AxisTrait,N:Copy,Num:NumTrait,T:Copy,K:Splitter>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:usize,splitter:&mut K)->DinoTree<A,N,BBox<Num,T>>{   

    #[repr(transparent)]
    pub struct SplitterWrapper<T>(
        pub T,
    );

    impl<T:Splitter> Splitter for SplitterWrapper<T>{
        fn div(&mut self)->Self{
            let a=self.0.div();
            SplitterWrapper(a)
        }
        fn add(&mut self,a:Self){
            self.0.add(a.0);
        }
        fn node_start(&mut self){self.0.node_start()}
        fn node_end(&mut self){self.0.node_end()}
    }        
    unsafe impl<T> Send for SplitterWrapper<T>{}
    unsafe impl<T> Sync for SplitterWrapper<T>{}

    let ss:&mut SplitterWrapper<K>=unsafe{std::mem::transmute(splitter)};
    new_inner(axis,n,bots,aabb_create,ss,height,par::Sequential,DefaultSorter)
    //(a,b.0)
}


///Returns Ok, then this tree's invariants are being met.
///Should always return true, unless the user corrupts the trees memory
///or if the contract of the HasAabb trait are not upheld.
#[inline]
pub fn are_invariants_met<A:AxisTrait,N:Copy,T:HasAabb+Copy>(tree:&DinoTree<A,N,T>)->Result<(),()> where T::Num:std::fmt::Debug{
    assert_invariants::are_invariants_met(tree)
}


///Return the fraction of bots that are in each level of the tree.
///The first element is the number of bots in the root level.
///The last number is the fraction in the lowest level of the tree.
///Ideally the fraction of bots in the lower level of the tree is high.
#[inline]
pub fn compute_tree_health<A:AxisTrait,N:Copy,T:HasAabb+Copy>(tree:&DinoTree<A,N,T>)->Vec<f64>{
    tree_health::compute_tree_health(tree)
}