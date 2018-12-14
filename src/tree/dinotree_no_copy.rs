
use super::*;


///A more advanced tree construction function where the use can choose, the height of the tree, the height at which to switch to sequential recursion, and a splitter callback (useful to measuring the time each level of the tree took, for example).
#[inline]
pub fn new_adv_no_copy<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy,K:Splitter+Send>(rebal_strat:Option<RebalStrat>,axis:A,n:N,bots:&'a mut[T],height:Option<usize>,splitter:&mut K,height_switch_seq:Option<usize>)->DinoTreeNoCopy<'a,A,N,T>{   
    //TODO make this the inner api????

    let height=match height{
        Some(height)=>height,
        None=>compute_tree_height_heuristic(bots.len())
    };

    let dlevel=compute_default_level_switch_sequential(height_switch_seq,height);
        

    let rebal_strat=match rebal_strat{
        Some(x)=>x,
        None=>RebalStrat::First
    };

    
    DinoTreeNoCopy::new_inner(rebal_strat,axis,n,bots,splitter,height,dlevel,DefaultSorter)    
}


///A more advanced tree construction function where the use can choose, the height of the tree, the height at which to switch to sequential recursion, and a splitter callback (useful to measuring the time each level of the tree took, for example).
#[inline]
pub fn new_adv_no_copy_seq<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy,K:Splitter+Send>(rebal_strat:Option<RebalStrat>,axis:A,n:N,bots:&'a mut[T],height:Option<usize>,splitter:&mut K)->DinoTreeNoCopy<'a,A,N,T>{   
    //TODO make this the inner api????

    let height=match height{
        Some(height)=>height,
        None=>compute_tree_height_heuristic(bots.len())
    };

    let rebal_strat=match rebal_strat{
        Some(x)=>x,
        None=>RebalStrat::First
    };

    
    DinoTreeNoCopy::new_inner(rebal_strat,axis,n,bots,splitter,height,par::Sequential,DefaultSorter)    
}


struct Index(u32);
impl reorder::HasIndex for Index{
    fn get(&self)->usize{
        self.0 as usize
    }
    fn set(&mut self,index:usize){
        self.0=index as u32;
    }
}


///A version where the bots are not copied. 
pub struct DinoTreeNoCopy<'a,A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:&'a mut [T],
    nodes:compt::dfs_order::CompleteTreeContainer<Node3<N,T>,compt::dfs_order::InOrder>,
    mover:Vec<Index>
}

impl<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTreeNoCopy<'a,A,N,T>{

    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    #[inline]
    pub fn new(axis:A,n:N,bots:&'a mut [T])->DinoTreeNoCopy<'a,A,N,T>{  
        new_adv_no_copy(None,axis,n,bots,None,&mut advanced::SplitterEmpty,None)
    }

    pub fn new_seq(axis:A,n:N,bots:&'a mut [T])->DinoTreeNoCopy<'a,A,N,T>{   
        new_adv_no_copy_seq(None,axis,n,bots,None,&mut advanced::SplitterEmpty)
    }

    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    pub fn into_original(mut self)->&'a mut [T]{
        reorder::reorder(self.bots,&mut self.mover)
    }

    pub(crate) fn new_inner<JJ:par::Joiner,K:Splitter+Send>(
        rebal_type:RebalStrat,axis:A,n:N,bots:&'a mut[T],ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTreeNoCopy<'a,A,N,T>
    {   
        let bots2=unsafe{&mut *(bots as *mut [_])};
        use tree::cont_tree::*;
        

        let num_bots=bots.len();
        let max=std::u32::MAX;
        
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let mut conts:Vec<_>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:*k.get(),index:index as u32}
        }).collect();


    
        
        let binstrat=match rebal_type{
            RebalStrat::First=>{
                BinStrat::LeftMidRight
            },
            RebalStrat::Second=>{
                BinStrat::MidLeftRight
            },
            RebalStrat::Third=>{
                BinStrat::LeftRightMid
            }
        };

        let mut cont_tree=ContTree::new(axis,par,&mut conts,sorter,ka,height,binstrat);




        let new_bots={
            impl<Num:NumTrait> reorder::HasIndex for Cont2<Num>{
                fn get(&self)->usize{
                    self.index as usize
                }
                fn set(&mut self,index:usize){
                    self.index=index as u32;
                }
            }
            //bots
            reorder::reorder(bots,cont_tree.get_conts_mut())
        };


        let new_tree={
            let new_nodes={
                let mut rest:Option<&mut [T]>=Some(new_bots);
                let mut new_nodes=Vec::with_capacity(cont_tree.get_tree().get_nodes().len());
                for node in cont_tree.get_tree_mut().dfs_inorder_iter(){
                    let (b,rest2)=rest.take().unwrap().split_at_mut(node.mid.len());
                    rest=Some(rest2);
                    new_nodes.push(Node3{n,fullcomp:node.fullcomp,mid:unsafe{std::ptr::Unique::new_unchecked(b as *mut [_])}});
                }
                new_nodes
            };

            let tree2=compt::dfs_order::CompleteTreeContainer::from_vec(new_nodes).unwrap();
            tree2
        };

        let mover=cont_tree.get_conts().iter().map(|a|Index(a.index)).collect();


        DinoTreeNoCopy{mover,axis,bots:bots2,nodes:new_tree}

    }

    pub fn as_ref_mut(&mut self)->DinoTreeRefMut<A,N,T>{
        DinoTreeRefMut{axis:self.axis,bots:self.bots,tree:&mut self.nodes}
    }
    pub fn as_ref(&self)->DinoTreeRef<A,N,T>{
        DinoTreeRef{axis:self.axis,bots:self.bots,tree:&self.nodes}
    }

}