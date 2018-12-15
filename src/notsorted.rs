use crate::inner_prelude::*;
use crate::advanced;


//Todo use this
pub struct NotSorted<A:AxisTrait,N,T:HasAabb>(pub DinoTree<A,N,T>);

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> NotSorted<A,N,BBox<Num,T>>{
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->NotSorted<A,N,BBox<Num,T>>{
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;


        let dlevel=advanced::compute_default_level_switch_sequential(None,height);
        
        NotSorted(DinoTree::new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,dlevel,NoSorter))
    }
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->NotSorted<A,N,BBox<Num,T>>{
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;

        let dlevel=par::Sequential;

        NotSorted(DinoTree::new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,dlevel,NoSorter))
    }

    pub fn new_adv_seq<K:Splitter>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:usize,splitter:&mut K)->NotSorted<A,N,BBox<Num,T>>{

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
        NotSorted(DinoTree::new_inner(RebalStrat::First,axis,n,bots,aabb_create,ss,height,par::Sequential,NoSorter))
    }
}
