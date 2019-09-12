


use crate::tree::*;
use crate::inner_prelude::*;


pub struct DinoTreeIndirect<'a,A: AxisTrait, N:NumTrait,T> {
    pub(crate) inner:DinoTreeInner<A,BBoxIndirect<'a,N,T>>,
}




pub struct DinoTreeIndirectBuilder<'a, A: AxisTrait,N:NumTrait,T> {
    axis: A,
    bots: &'a mut [BBox<N,T>],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}





impl<'a, A: AxisTrait, N:NumTrait,T:Send+Sync> DinoTreeIndirectBuilder<'a, A, N,T> {
    
    ///Build in parallel
    pub fn build_par(self) -> DinoTreeIndirect<'a,A,N,T> {

        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        
        let mut conts: Vec<_> = self.bots
            .iter_mut().map(|inner|BBoxIndirect{inner}).collect();

        let cont_tree = create_tree_par(self.axis, dlevel,&mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,conts,cont_tree)
    }
}

impl<'a, A: AxisTrait, N:NumTrait,T> DinoTreeIndirectBuilder<'a, A, N,T> {

    pub fn new(axis: A, bots: &'a mut [BBox<N,T>]) -> DinoTreeIndirectBuilder<'a, A,N,T> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeIndirectBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }

    pub fn build_seq(self) -> DinoTreeIndirect<'a, A,N, T> {
        
        let mut conts: Vec<_> = self.bots
            .iter_mut().map(|inner|BBoxIndirect{inner}).collect();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,conts,cont_tree)
        
    }

    
    fn tree_finish(axis:A,
        conts:Vec<BBoxIndirect<'a,N,T>>,
        tree:compt::dfs_order::CompleteTreeContainer<Node<BBoxIndirect<'a,N,T>>,
        compt::dfs_order::PreOrder>) -> DinoTreeIndirect<'a,A,N,T>{

        DinoTreeIndirect{inner:DinoTreeInner{axis,bots:conts,tree}}
    }
}




impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefTrait for DinoTreeIndirect<'a,A,N,T>{
    type Item=BBoxIndirect<'a,N,T>;
    type Axis=A;
    type Num=N;
    //type Inner=T;
    
    #[inline(always)]
    fn axis(&self)->Self::Axis{
        self.inner.axis
    }
    #[inline(always)]
    fn vistr(&self)->Vistr<Self::Item>{
        Vistr {
            inner: self.inner.tree.vistr(),
        }
    }

    ///Return the height of the dinotree.
    #[inline(always)]
    fn height(&self) -> usize
    {
        self.inner.tree.get_height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline(always)]
    fn num_nodes(&self) -> usize
    {
        self.inner.tree.get_nodes().len()
    }

    ///Return the number of bots in the tree.
    #[inline(always)]
    fn num_bots(&self) -> usize
    {
        self.inner.bots.len()
    }

}


impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefMutTrait for DinoTreeIndirect<'a,A,N,T>{ 
    #[inline(always)]   
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.inner.tree.vistr_mut(),
        }
    }
}
