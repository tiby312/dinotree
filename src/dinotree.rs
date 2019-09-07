use crate::tree::*;
use crate::inner_prelude::*;


#[repr(transparent)]
pub struct DinoTree<'a,A: AxisTrait, N:NumTrait,T> {
    pub(crate) inner:DinoTreeInner<A,BBoxMut<'a,N,T>>,
}


impl<'a,A:AxisTrait,N:NumTrait,T> DinoTree<'a,A,N,T>{
    #[inline(always)]
    pub fn get_bots_mut(&mut self)->ElemSliceMut<BBoxMut<'a,N,T>>{
        ElemSliceMut::new(ElemSlice::from_slice_mut(&mut self.inner.bots))
    }
    #[inline(always)]
    pub fn get_bots(&self)->&ElemSlice<BBoxMut<'a,N, T>>{
        ElemSlice::from_slice(&self.inner.bots)
    }
}

impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefTrait for DinoTree<'a,A,N,T>{
    type Item=BBoxMut<'a,N,T>;
    type Axis=A;
    type Num=N;
    type Inner=T;
    
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


impl<'a,A:AxisTrait,N:NumTrait+'a,T:'a> DinoTreeRefMutTrait for DinoTree<'a,A,N,T>{  
    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.inner.tree.vistr_mut(),
        }
    }
}


pub struct DinoTreeBuilder<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>> {
    pub(crate) axis: A,
    pub(crate) bots: &'a mut [T],
    pub(crate) aabb_create: F,
    pub(crate) rebal_strat: BinStrat,
    pub(crate) height: usize,
    pub(crate) height_switch_seq: usize,
}


impl<'a, A: AxisTrait, T: Send+Sync, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeBuilder<'a, A, T, Num, F>
{
    
    pub fn build_par(&mut self) -> DinoTree<'a,A,Num,T> {

        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);

        let mut conts=self.tree_prep();

        let cont_tree = create_tree_par(self.axis, dlevel,&mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        self.tree_finish(conts,cont_tree)
    }
}

impl<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeBuilder<'a, A, T, Num, F>
{
    pub fn new(axis: A, bots: &mut [T], aabb_create: F) -> DinoTreeBuilder<A, T, Num, F> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeBuilder {
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
        &mut self,
        splitter: &mut S,
    ) -> DinoTree<'a,A,Num,T> {

        let mut conts=self.tree_prep();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, splitter, self.height, self.rebal_strat);

        self.tree_finish(conts,cont_tree)
    }

    ///Build sequentially.
    pub fn build_seq(&mut self) -> DinoTree<'a,A,Num,T> {
        let mut conts=self.tree_prep();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        self.tree_finish(conts,cont_tree)
    }


    pub(crate) fn tree_prep(&mut self)->Vec<BBoxMut<'a,Num,T>>{

        let bots:&mut [T]=core::mem::replace::<&mut [T]>(&mut self.bots,&mut []);
        let aabb_create = &mut self.aabb_create;
        
        bots
            .iter_mut()
            .map(move |k| BBoxMut::new(aabb_create(k),k))
            .collect()
    }
    
    pub(crate) fn tree_finish(&self,
        conts:Vec<BBoxMut<'a,Num,T>>,
        cont_tree:compt::dfs_order::CompleteTreeContainer<Node<BBoxMut<'a,Num,T>>,
        compt::dfs_order::PreOrder>) -> DinoTree<'a,A,Num,T>{

        DinoTree{
            inner:DinoTreeInner{
                axis:self.axis,
                bots:conts,
                tree:cont_tree
            }
        }
    }
}

 

