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
