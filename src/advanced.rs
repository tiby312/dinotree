
/*
///Converts an iterator and a aabb creating function into a Vec of BBox
pub fn into_bbox_vec<I:Iterator,Num:NumTrait>(a:I,mut b:impl FnMut(&I::Item)->Rect<Num>)->Vec<BBox<Num,I::Item>>{
    a.map(|a|unsafe{BBox::new(b(&a),a)}).collect()
}
*/



pub use crate::tree::compute_default_level_switch_sequential;
pub use crate::tree::compute_tree_height_heuristic;
pub use crate::tree::compute_tree_height_heuristic_debug;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);

    ///Called at the start of the recursive call.
    fn node_start(&mut self);

    ///It is important to note that this gets called in other places besides in the final recursive call of a leaf.
    ///They may get called in a non leaf if its found that there is no more need to recurse further.
    fn node_end(&mut self);
}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty {
    fn div(&mut self) -> Self {
        SplitterEmpty
    }
    fn add(&mut self, _: Self) {}
    fn node_start(&mut self) {}
    fn node_end(&mut self) {}
}

pub use crate::oned::sweeper_update;
pub use crate::tree::default_level_switch_sequential;
pub use crate::tree::BinStrat;
pub use crate::tools::Unique;


///Contains code to write generic code that can be run in parallel, or sequentially. Not intended to be used directly by the user.
///Used by algorithms that operate on the tree.
pub mod par{
    use compt::Depth;

    ///Indicates if we are low enough into the tree that we should switch to a sequential version of the
    ///algorithm.
    pub trait Joiner: Send + Sync + Copy + Clone {
        fn into_seq(&self) -> Sequential;
        fn should_switch_to_sequential(&self, a: Depth) -> bool;
    }

    ///Indicates that an algorithm should run in parallel up until
    ///the specified depth.
    #[derive(Copy, Clone)]
    pub struct Parallel(pub Depth);
    impl Parallel {
        ///The height at which to switch to sequential.
        pub fn new(d: Depth) -> Self {
            Parallel(d)
        }
    }
    impl Joiner for Parallel {
        fn into_seq(&self) -> Sequential {
            Sequential
        }

        fn should_switch_to_sequential(&self, a: Depth) -> bool {
            a.0 >= (self.0).0
        }
    }

    ///Indicates that an algorithm should run sequentially.
    ///Once we transition to sequential, we always want to recurse sequentially.
    #[derive(Copy, Clone)]
    pub struct Sequential;
    impl Joiner for Sequential {
        fn into_seq(&self) -> Sequential {
            Sequential
        }

        fn should_switch_to_sequential(&self, _a: Depth) -> bool {
            true
        }
    }

}