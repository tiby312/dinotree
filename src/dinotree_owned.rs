/*
use crate::tree::*;
use crate::inner_prelude::*;


///TODO implement this
pub struct DinoTreeOwned<A: AxisTrait, N:NumTrait,T> {
    inner:DinoTreeInner<A,BBoxPtr<N,T>>,
    bots:Vec<T>
}
impl<A:AxisTrait,N:NumTrait,T> DinoTreeOwned<A,N,T>{

    #[inline(always)]
    pub fn get_aabb_bots_mut(&mut self)->ProtectedBBoxSlice<BBoxPtr<N,T>>{
        ProtectedBBoxSlice::new(&mut self.inner.bots)
    }

    #[inline(always)]
    pub fn get_aabb_bots(&self)->&[BBoxPtr<N, T>]{
        &self.inner.bots
    }


    #[inline(always)]
    pub fn get_bots_mut(&mut self)->&mut [T]{
        &mut self.bots
    }

    #[inline(always)]
    pub fn get_bots(&self)->&[T]{
        &self.bots
    }

    pub fn into_inner(self)->Vec<T>{
        self.bots
    }
}

impl<A:AxisTrait,N:NumTrait,T> DinoTreeRefTrait for DinoTreeOwned<A,N,T>{
    type Item=BBoxPtr<N,T>;
    type Axis=A;
    type Num=N;
    
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

    #[inline(always)]
    fn height(&self) -> usize
    {
        self.inner.tree.get_height()
    }

    #[inline(always)]
    fn num_nodes(&self) -> usize
    {
        self.inner.tree.get_nodes().len()
    }

    #[inline(always)]
    fn num_bots(&self) -> usize
    {
        self.inner.bots.len()
    }

}


impl<A:AxisTrait,N:NumTrait,T> DinoTreeRefMutTrait for DinoTreeOwned<A,N,T>{  
    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.inner.tree.vistr_mut(),
        }
    }
}






pub struct DinoTreeOwnedBuilder<A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>> {
    pub(crate) axis: A,
    pub(crate) bots: Vec<T>,
    pub(crate) aabb_create: F,
    pub(crate) rebal_strat: BinStrat,
    pub(crate) height: usize,
    pub(crate) height_switch_seq: usize,
}


impl<A: AxisTrait, T: Send+Sync, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeOwnedBuilder<A, T, Num, F>
{
    
    pub fn build_par(mut self) -> DinoTreeOwned<A,Num,T> {

        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);

        let mut conts=self.tree_prep();

        let cont_tree = create_tree_par(self.axis, dlevel,&mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,self.bots,conts,cont_tree)
    }
}

impl<A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeOwnedBuilder<A, T, Num, F>
{
    pub fn new(axis: A, bots: Vec<T>, aabb_create: F) -> DinoTreeOwnedBuilder<A, T, Num, F> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeOwnedBuilder {
            axis,
            bots,
            aabb_create,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }

    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    #[inline(always)]
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    #[inline(always)]
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.height_switch_seq = height;
        self
    }

    ///Build with a Splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        mut self,
        splitter: &mut S,
    ) -> DinoTreeOwned<A,Num,T> {

        let mut conts=self.tree_prep();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, splitter, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,self.bots,conts,cont_tree)
    }

    ///Build sequentially.
    pub fn build_seq(mut self) -> DinoTreeOwned<A,Num,T> {
        let mut conts=self.tree_prep();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,self.bots,conts,cont_tree)
    }


    pub(crate) fn tree_prep(&mut self)->Vec<BBoxPtr<Num,T>>{

        let aabb_create = &mut self.aabb_create;
        
        self.bots
            .iter_mut()
            .map(move |k| unsafe{BBoxPtr::new(aabb_create(k),k)})
            .collect()
    }
    
    pub(crate) fn tree_finish(
        axis:A,
        bots:Vec<T>,
        conts:Vec<BBoxPtr<Num,T>>,
        cont_tree:compt::dfs_order::CompleteTreeContainer<Node<BBoxPtr<Num,T>>,
        compt::dfs_order::PreOrder>) -> DinoTreeOwned<A,Num,T>{

        DinoTreeOwned{
            inner:DinoTreeInner{
                axis,
                bots:conts,
                tree:cont_tree,
            },
            bots
        }
    }
}

 

*/