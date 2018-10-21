
use inner_prelude::*;
use axgeom::Rect;

use dyntree::fast_alloc;

pub fn new_adv<A:AxisTrait,N:Copy,T:HasAabb+Copy,K:Splitter+Send>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<T::Num>,height:usize,splitter:K,height_switch_seq:usize)->(DynTree<A,N,BBox<T::Num,T>>,K){   
    //let height=heur.compute_tree_height_heuristic(bots.len()); 
    //let ka=TreeTimer2::new(height);


    //on xps13 5 seems good
    //const DEPTH_SEQ:usize=6;

    let gg=if height<=height_switch_seq{
        0
    }else{
        height-height_switch_seq
    };
    
    let dlevel=par::Parallel::new(Depth(gg));


    let a=fast_alloc::new(axis,n,bots,aabb_create,splitter,height,dlevel);
    a
    //(a.0,(a.1).into_iter())
}

pub fn new_adv_seq<A:AxisTrait,N:Copy,T:HasAabb+Copy,K:Splitter>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<T::Num>,height:usize,splitter:K)->(DynTree<A,N,BBox<T::Num,T>>,K){   

    pub struct SplitterWrapper<T>(
        pub T,
    );

    impl<T:Splitter> Splitter for SplitterWrapper<T>{
        fn div(self)->(Self,Self){
            let (a,b)=self.0.div();
            (SplitterWrapper(a),SplitterWrapper(b))
        }
        fn add(self,a:Self)->Self{
            let a=self.0.add(a.0);
            SplitterWrapper(a)
        }
        fn node_start(&mut self){self.0.node_start()}
        fn node_end(&mut self){self.0.node_end()}
    }        
    unsafe impl<T> Send for SplitterWrapper<T>{}
    unsafe impl<T> Sync for SplitterWrapper<T>{}


    let (a,b)=fast_alloc::new(axis,n,bots,aabb_create,SplitterWrapper(splitter),height,par::Sequential);
    (a,b.0)
}


///Returns Ok, then this tree's invariants are being met.
///Should always return true, unless the user corrupts the trees memory
///or if the contract of the HasAabb trait are not upheld.
pub fn are_invariants_met<A:AxisTrait,N:Copy,T:HasAabb+Copy>(tree:&DynTree<A,N,T>)->Result<(),()> where T::Num:std::fmt::Debug{
    assert_invariants::are_invariants_met(tree)
}


///Return the fraction of bots that are in each level of the tree.
///The first element is the number of bots in the root level.
///The last number is the fraction in the lowest level of the tree.
///Ideally the fraction of bots in the lower level of the tree is high.
pub fn compute_tree_health<A:AxisTrait,N:Copy,T:HasAabb+Copy>(tree:&DynTree<A,N,T>)->tree_health::LevelRatioIterator<N,T>{
    tree_health::compute_tree_health(tree)
}