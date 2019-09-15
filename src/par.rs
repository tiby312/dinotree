use compt::Depth;




pub const SWITCH_SEQUENTIAL_DEFAULT:usize = 6;

///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn compute_level_switch_sequential(depth: usize, height: usize) -> Parallel {
    //const DEPTH_SEQ:usize=4;
    let dd = depth;

    let gg = if height <= dd { 0 } else { height - dd };

    Parallel::new(Depth(gg))
}



pub enum ParResult<X,Y>{
    Parallel([X;2]),
    Sequential([Y;2])
}

pub trait Joiner:Sized+Send+Sync{
    fn next(self,a:Depth)->ParResult<Self,Sequential>;
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
    fn next(self,a:Depth)->ParResult<Self,Sequential>{
        if a.0 >= ((self.0).0){
            ParResult::Sequential([Sequential,Sequential])
        }else{
            ParResult::Parallel([Parallel(a),Parallel(a)])
        }
    }
}

///Indicates that an algorithm should run sequentially.
///Once we transition to sequential, we always want to recurse sequentially.
#[derive(Copy, Clone)]
pub struct Sequential;
impl Joiner for Sequential {
    fn next(self,_:Depth)->ParResult<Self,Sequential>{
        ParResult::Sequential([Sequential,Sequential])
    }
}
