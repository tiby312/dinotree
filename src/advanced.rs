use crate::inner_prelude::*;
pub use crate::tree::RebalStrat;

fn into_secs(elapsed:std::time::Duration)->f64{
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)   
}

///Measure the time each level of a recursive algorithm takes that supports the Splitter trait.
///Note that the number of elements in the returned Vec could be less than the height of the tree.
///This can happen if the recursive algorithm does not recurse all the way to the leafs because it
///deemed it not necessary.
#[derive(Default)]
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



pub use crate::tree::compute_tree_height_heuristic_debug;
pub use crate::tree::compute_default_level_switch_sequential;
pub use crate::tree::compute_tree_height_heuristic;


///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter:Sized{

    ///Called to split this into two to be passed to the children.
    fn div(&mut self)->Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self,b:Self);

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



pub use crate::tree::dinotree::DinoTree;
pub use crate::tree::dinotree_no_copy::DinoTreeNoCopy;
pub use crate::notsorted::NotSorted;
pub use crate::oned::sweeper_update;

//pub use crate::tree::dinotree::new_adv;
//pub use crate::tree::dinotree::new_adv_seq;
//pub use crate::tree::dinotree_no_copy::new_adv_no_copy;
//pub use crate::tree::dinotree_no_copy::new_adv_no_copy_seq;
pub use crate::tree::default_level_switch_sequential;




pub use crate::assert_invariants::are_invariants_met;

