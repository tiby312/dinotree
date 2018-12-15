use crate::inner_prelude::*;




///The datastructure this crate revolves around.
pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:Vec<T>,
    tree:compt::dfs_order::CompleteTreeContainer<Node3<N,T>,compt::dfs_order::InOrder>,
    mover:Vec<u32>
}

///Provides many of the same arguments as new_adv, with the exception of the height at which to switch to sequential, since this is already sequential.
pub fn new_adv_seq<A:AxisTrait,N:Copy,Num:NumTrait,T:Copy,K:Splitter>(rebal_strat:Option<RebalStrat>,axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:Option<usize>,splitter:&mut K)->DinoTree<A,N,BBox<Num,T>>{   
    let height=match height{
        Some(height)=>height,
        None=>compute_tree_height_heuristic(bots.len())
    };


    let rebal_strat=match rebal_strat{
        Some(x)=>x,
        None=>RebalStrat::First
    };


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
    DinoTree::new_inner(rebal_strat,axis,n,bots,aabb_create,ss,height,par::Sequential,DefaultSorter)
    //(a,b.0)
}
///A more advanced tree construction function where the use can choose, the height of the tree, the height at which to switch to sequential recursion, and a splitter callback (useful to measuring the time each level of the tree took, for example).
pub fn new_adv<A:AxisTrait,N:Copy,Num:NumTrait,T:Copy,K:Splitter+Send>(rebal_strat:Option<RebalStrat>,axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:Option<usize>,splitter:&mut K,height_switch_seq:Option<usize>)->DinoTree<A,N,BBox<Num,T>>{   
    
    let height=match height{
        Some(height)=>height,
        None=>compute_tree_height_heuristic(bots.len())
    };

    let dlevel=compute_default_level_switch_sequential(height_switch_seq,height);
        

    let rebal_strat=match rebal_strat{
        Some(x)=>x,
        None=>RebalStrat::First
    };

    
    DinoTree::new_inner(rebal_strat,axis,n,bots,aabb_create,splitter,height,dlevel,DefaultSorter)    
}


impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{

    pub(crate) fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
	    rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree<A,N,BBox<Num,T>>
	{   
        use crate::tree::cont_tree::*;
            

        let num_bots=bots.len();
        let max=std::u32::MAX;
        
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");

        let mut conts:Vec<_>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
        }).collect();
        
        let (new_bots,new_tree)={
            let binstrat=match rebal_type{
                RebalStrat::First=>{
                    BinStrat::LeftMidRight
                },
                RebalStrat::Second=>{
                    //BinStrat::MidLeftRight
                    BinStrat::LeftMidRightUnchecked
                },
                RebalStrat::Third=>{
                    BinStrat::LeftRightMid
                }
            };

            let mut cont_tree=ContTree::new(axis,par,&mut conts,sorter,ka,height,binstrat);

            let mut new_bots:Vec<_>=cont_tree.get_conts().iter().map(|a|BBox{rect:a.rect,inner:*unsafe{bots.get_unchecked(a.index as usize)}}).collect();            

            let new_nodes={
                let mut rest:Option<&mut [BBox<Num,T>]>=Some(&mut new_bots);
                let mut new_nodes=Vec::with_capacity(cont_tree.get_tree().get_nodes().len());
                for node in cont_tree.get_tree_mut().dfs_inorder_iter(){
                    let (b,rest2)=rest.take().unwrap().split_at_mut(node.mid.len());
                    rest=Some(rest2);
                    let b=unsafe{std::ptr::Unique::new_unchecked(b as *mut [_])};
                    new_nodes.push(Node3{n,fullcomp:node.fullcomp,mid:b});
                }
                new_nodes
            };

            (new_bots,compt::dfs_order::CompleteTreeContainer::from_vec(new_nodes).unwrap())
        };

        let mover=conts.drain(..).map(|a|a.index).collect();

        DinoTree{mover,axis,bots:new_bots,tree:new_tree}
        
        
	}

    
    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{  
        new_adv(None,axis,n,bots,aabb_create,None,&mut advanced::SplitterEmpty,None,)
    }

    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        new_adv_seq(None,axis,n,bots,aabb_create,None,&mut advanced::SplitterEmpty)
    }
    
}


impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{
    pub fn as_ref_mut(&mut self)->DinoTreeRefMut<A,N,T>{
        DinoTreeRefMut{axis:self.axis,bots:&mut self.bots,tree:&mut self.tree}
    }
    pub fn as_ref(&self)->DinoTreeRef<A,N,T>{
        DinoTreeRef{axis:self.axis,bots:&self.bots,tree:&self.tree}
    }
    
    pub fn with_extra<N2:Copy>(self,n2:N2)->DinoTree<A,N2,T>{
        let mut old_nodes=self.tree.into_nodes();
        let new_nodes=old_nodes.drain(..).map(|node|{
            Node3{n:n2,fullcomp:node.fullcomp,mid:node.mid}
        }).collect();
        let new_tree=compt::dfs_order::CompleteTreeContainer::from_vec(new_nodes).unwrap();
        DinoTree{axis:self.axis,bots:self.bots,tree:new_tree,mover:self.mover}
    }
    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    pub fn apply<X>(&self,bots:&mut [X],conv:impl Fn(&T,&mut X)){
        assert_eq!(bots.len(),self.bots.len());
        for (bot,mov) in self.bots.iter().zip_eq(self.mover.iter()){
            let target=unsafe{bots.get_unchecked_mut(*mov as usize)};
            conv(bot,target);
        }
    }

    ///Apply changes to the internals of the bots (not the aabb) back into the tree without recreating the tree.
    pub fn apply_into<X>(&mut self,bots:&[X],conv:impl Fn(&X,&mut T)){
        
        assert_eq!(bots.len(),self.bots.len());

        let treev=self.bots.iter_mut();
        
        for (bot,mov) in treev.zip_eq(self.mover.iter()){
            let source=unsafe{bots.get_unchecked(*mov as usize)};
            conv(source,bot)
        }
        
    }

}

