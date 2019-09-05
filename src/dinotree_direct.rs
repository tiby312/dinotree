


use crate::tree::*;
use crate::inner_prelude::*;

use crate::dinotree::DinoTreeInner;


///Version of dinotree that makes a copy of all the elements.
pub struct DinoTreeDirect<'a,A: AxisTrait, N:NumTrait,T> {
    pub(crate) inner:DinoTreeInner<A,BBox<N,T>>,
    pub(crate) index:Vec<u32>
}





///Builder for a DinoTree
pub struct DinoTreeDirectBuilder<'a, A: AxisTrait,N:NumTrait,T> {
    axis: A,
    bots: &'a mut [BBox<N,T>],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}

impl<'a, A: AxisTrait, N:NumTrait,T> DinoTreeDirectBuilder<'a, A, N,T> {
    #[inline]
    pub fn new(axis: A, bots: &'a mut [BBox<N,T>]) -> DinoTreeDirectBuilder<'a, A,N,T> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeDirectBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }



    #[inline]
    pub fn build_seq(self) -> DinoTreeDirect<'a, A,N, T> {
        unimplemented!();
    }

    
    fn tree_prep(&mut self)->Vec<BBox<N,u32>>{
        let axis = self.axis;
        
        let mut conts: Vec<_> = self.bots
            .iter_mut().enumerate().map(|inner|BBox::new(inner.rect,index as u32)).collect();
        conts
    }
    fn tree_finish(axis:A,
        conts:Vec<BBox<N,u32>>,
        tree:compt::dfs_order::CompleteTreeContainer<Node<BBox<N,T>>,
        compt::dfs_order::PreOrder>) -> DinoTreeDirect<A,N,T>{

        DinoTreeIndirect{inner:DinoTreeInner{axis,bots:conts,tree}}
    }
}


/*

impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefTrait for DinoTreeIndirect<'a,A,N,T>{
    type Item=BBoxIndirect<'a,N,T>;
    type Axis=A;
    type Num=N;
    type Inner=T;
    
    fn axis(&self)->Self::Axis{
        self.inner.axis
    }
    fn vistr(&self)->Vistr<Self::Item>{
        
        Vistr {
            inner: self.inner.tree.vistr(),
        }
        
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        self.inner.tree.get_height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        self.inner.tree.get_nodes().len()
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        self.inner.bots.len()
    }

}


impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefMutTrait for DinoTreeIndirect<'a,A,N,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.inner.tree.vistr_mut(),
        }
    }
}

*//