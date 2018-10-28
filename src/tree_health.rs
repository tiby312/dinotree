use super::*;
use axgeom::AxisTrait;
use compt::Visitor;

///Outputs the ratio of the number of bots at the current level compared to the total number of bots in the tree.
///Starts at the root level and ends with the leaf level.
pub struct LevelRatioIterator<'a,N:'a,T:HasAabb+'a>{
    height:usize,
    total_bots:usize,
    itt:compt::BfsIter<compt::LevelIter<Vistr<'a,N,T>>>,
    acc:usize,
    prev_depth:compt::Depth
}

impl<'a,N:'a,T:HasAabb+'a> std::iter::FusedIterator for LevelRatioIterator<'a,N,T>{}
unsafe impl<'a,N:'a,T:HasAabb+'a> std::iter::TrustedLen for LevelRatioIterator<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Iterator for LevelRatioIterator<'a,N,T>{
    type Item=f64;
    fn next(&mut self)->Option<Self::Item>{
        for ((depth,node),_extra) in &mut self.itt{
            self.acc+=node.range.len();
            if depth.0!=self.prev_depth.0{
                let ret=self.acc;
                self.acc=0;
                self.prev_depth=depth;
                return Some(ret as f64/self.total_bots as f64)
            }
            self.prev_depth=depth;
        }

        if self.acc!=0{
            let ret=self.acc;
            self.acc=0;
            return Some(ret as f64/self.total_bots as f64)
        }else{
            None
        }
    }
    fn size_hint(&self)->(usize,Option<usize>){
        let length=self.height-self.prev_depth.0;
        (length,Some(length))
    }
}

///Returns the fraction of bots that are in each level of the tree.
pub fn compute_tree_health<A:AxisTrait,N,T:HasAabb>(tree:&DinoTree<A,N,T>)->LevelRatioIterator<N,T>{
    let itt=tree.vistr().with_depth(compt::Depth(0)).bfs_iter();
    let height=tree.height();
    let total_bots=tree.num_bots();
    LevelRatioIterator{total_bots,height,itt,acc:0,prev_depth:compt::Depth(42)}
}
