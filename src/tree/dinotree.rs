use crate::inner_prelude::*;
use super::notsorted::*;

///The datastructure this crate revolves around.
pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:Vec<T>,
    tree:compt::dfs_order::CompleteTreeContainer<Node3<N,T>,compt::dfs_order::InOrder>,
    mover:Vec<u32>
}


///Builder for a DinoTree
pub struct DinoTreeBuilder<'a,A:AxisTrait,N,T,Num:NumTrait,F:FnMut(&T)->Rect<Num>>{
    axis:A,
    n:N,
    bots:&'a [T],
    aabb_create:F,
    rebal_strat:RebalStrat,
    height:usize,
    height_switch_seq:usize
}

impl<'a,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait,F:FnMut(&T)->Rect<Num>> DinoTreeBuilder<'a,A,N,T,Num,F>{
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:F)->DinoTreeBuilder<A,N,T,Num,F>{
        let rebal_strat=RebalStrat::First;
        let height=compute_tree_height_heuristic(bots.len());
        let height_switch_seq=default_level_switch_sequential();

        DinoTreeBuilder{axis,n,bots,aabb_create,rebal_strat,height,height_switch_seq}
    }

    ///Choose a custom rebalance stratagy.
    pub fn with_rebal_strat(&mut self,strat:RebalStrat)->&mut Self{
        self.rebal_strat=strat;
        self
    }

    ///Choose a custom height for the tree.
    pub fn with_height(&mut self,height:usize)->&mut Self{
        self.height=height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    pub fn with_height_switch_seq(&mut self,height:usize)->&mut Self{
        self.height_switch_seq=height;
        self
    }


    pub fn build_with_splitter_seq<S:Splitter>(&mut self,splitter:&mut S)->DinoTree<A,N,BBox<Num,T>>{

        #[repr(transparent)]
        pub struct SplitterWrap<S>{
            inner:S
        }
        impl<S:Splitter> Splitter for SplitterWrap<S>{
            fn div(&mut self)->Self{
                SplitterWrap{inner:self.inner.div()}
            }
            fn add(&mut self,a:Self){
                self.inner.add(a.inner)
            }
            fn node_start(&mut self){
                self.inner.node_start();
            }
            fn node_end(&mut self){
                self.inner.node_end()
            }
        }
        unsafe impl<S> Send for SplitterWrap<S>{}

        let splitter:&mut SplitterWrap<S>=unsafe{&mut *((splitter as *mut S) as *mut SplitterWrap<S>)};


        self.build_inner(par::Sequential,DefaultSorter,splitter)
    }

    pub fn build_seq(&mut self)->DinoTree<A,N,BBox<Num,T>>{
        self.build_inner(par::Sequential,DefaultSorter,&mut crate::advanced::SplitterEmpty)
    }

    pub fn build_par(&mut self)->DinoTree<A,N,BBox<Num,T>>{
        let dlevel=compute_default_level_switch_sequential(self.height_switch_seq,self.height);
        self.build_inner(dlevel,DefaultSorter,&mut crate::advanced::SplitterEmpty)
    }


    pub fn build_not_sorted_seq(&mut self)->NotSorted<A,N,BBox<Num,T>>{
        NotSorted(self.build_inner(par::Sequential,NoSorter,&mut crate::advanced::SplitterEmpty))
    }

    pub fn build_not_sorted_par(&mut self)->NotSorted<A,N,BBox<Num,T>>{
        let dlevel=compute_default_level_switch_sequential(self.height_switch_seq,self.height);
        NotSorted(self.build_inner(dlevel,NoSorter,&mut crate::advanced::SplitterEmpty))
    }


    fn build_inner<JJ:par::Joiner,S:Splitter+Send>(&mut self,par:JJ,sorter:impl Sorter,ka:&mut S)->DinoTree<A,N,BBox<Num,T>>{
        use crate::tree::cont_tree::*;


        let bots=self.bots;
        let axis=self.axis;
        let aabb_create=&mut self.aabb_create;
        let n=self.n;

        let height=self.height;
        let rebal_type=self.rebal_strat;
            




        let num_bots=bots.len();
        let max=std::u32::MAX;
        
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported",max);

        let mut conts:Vec<_>=bots.iter().enumerate().map(move |(index,k)|{
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

