use crate::tree::*;
use crate::inner_prelude::*;





///Version of dinotree that makes a copy of all the elements.
pub(crate) struct DinoTreeInner<A: AxisTrait, T: HasAabbMut> {
    pub axis: A,
    pub bots: Vec<T>,
    pub tree: compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>,
}

pub struct DinoTreeOwned<A:AxisTrait,N:NumTrait,T>{
    tree:DinoTreeInner<A,BBoxPtr<N,T>>,
    bots:Vec<T>
}

impl<A:AxisTrait,N:NumTrait,T> DinoTreeOwned<A,N,T>{


    #[inline(always)]
    pub fn into_inner(self)->Vec<T>{
        let DinoTreeOwned{tree:_,bots}=self;
        bots
    }

    /*
    #[inline(always)]
    pub fn rebuild(&mut self,func:impl FnMut(Rect<N>,&T)->Rect<N>){
        unimplemented!()       
    }
    */

    #[inline(always)]
    pub fn as_mut(&mut self)->&mut DinoTree<A,N,T>{
        let a=(&mut self.tree) as *mut _;
        unsafe{&mut *(a as *mut _)}
    }

    #[inline(always)]
    pub fn as_ref(&self)->&DinoTree<A,N,T>{
        let a=(&self.tree) as *const _;
        unsafe{&*(a as *const _)}
    }
}

///Version of dinotree that makes a copy of all the elements.
#[repr(transparent)]
pub struct DinoTree<'a,A: AxisTrait, N:NumTrait,T> {
    pub(crate) inner:DinoTreeInner<A,BBoxMut<'a,N,T>>,
}


impl<'a,A:AxisTrait,N:NumTrait,T> DinoTree<'a,A,N,T>{
    pub fn get_bots_mut(&mut self)->ElemSliceMut<BBoxMut<'a,N,T>>{
        ElemSliceMut::new(ElemSlice::from_slice_mut(&mut self.inner.bots))
    }
    pub fn get_bots(&self)->&ElemSlice<BBoxMut<'a,N,T>>{
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


impl<'a,A:AxisTrait,N:NumTrait+'a,T:'a> DinoTreeRefMutTrait for DinoTree<'a,A,N,T>{  
    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.inner.tree.vistr_mut(),
        }
    }
}



pub struct DinoTreeOwnedBuilder<A:AxisTrait,T,Num:NumTrait,F:FnMut(&T) -> Rect<Num>>{
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
    
    ///Build in parallel
    #[inline(always)]
    pub fn build_par(self) -> DinoTreeOwned<A,Num,T> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut SplitterEmpty)
    }
}

impl<A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeOwnedBuilder<A, T, Num, F>
{
    ///Create a dinotree builder.
    ///The user picks the axis along which the first divider will partition.
    ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
    ///The user also passes a function to create the bounding box of each bot in the slice passed.
    #[inline(always)]
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




    ///Choose a custom bin stratagy.
    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    ///Choose a custom height for the tree.
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

    
    ///Build sequentially.
    #[inline(always)]
    pub fn build_seq(self) -> DinoTreeOwned<A,Num,T> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut SplitterEmpty,
        )
    }

    pub(crate) fn build_inner<JJ: par::Joiner, S: Splitter + Send>(
        self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut S,
    ) ->  DinoTreeOwned<A,Num,T>{

        let DinoTreeOwnedBuilder{axis,mut bots,mut aabb_create,rebal_strat,height,height_switch_seq:_}=self;


        let binstrat = rebal_strat;

        let num_bots = bots.len();
        let max = core::u32::MAX;

        assert!(
            num_bots < max as usize,
            "problems of size {} are bigger are not supported",
            max
        );

        let mut conts: Vec<_> = bots
            .iter_mut()
            .map(move |k| unsafe{BBoxSendSync::new(aabb_create(k),k)})
            .collect();

        let (new_bots, new_tree) = {
            let cont_tree = create_tree(axis, par, &mut conts, sorter, ka, height, binstrat);

            let mut new_bots: Vec<_> = conts.drain(..)
                .map(|a| {
                    let (rect,inner)=a.into_inner();
                    BBoxPtr::new(rect,tools::Unique::new(inner).unwrap())
                })
                .collect();


            let new_nodes = {
                let mut rest: Option<&mut [BBoxPtr<Num, _>]> = Some(&mut new_bots);
                let mut new_nodes = Vec::with_capacity(cont_tree.get_nodes().len());

                
                for node in cont_tree.get_nodes().iter() {
                    let (b, rest2) = rest.take().unwrap().split_at_mut(node.get().bots.len());
                    rest = Some(rest2);
                    let b = tools::Unique::new(ElemSlice::from_slice_mut(b) as *mut _).unwrap();
                    new_nodes.push(Node {
                        range: b,
                        cont: node.cont,
                        div: node.div,
                    })
                }
                new_nodes
            };

            (
                new_bots,
                compt::dfs_order::CompleteTreeContainer::from_preorder(new_nodes).unwrap(),
            )
        };

        let tree=DinoTreeInner {
            axis,
            bots: new_bots,
            tree: new_tree,
        };
        
        DinoTreeOwned{
            tree,
            bots
        }
    }
}

///Builder for a DinoTree
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree::copy::DinoTreeBuilder;
/// use dinotree_sample::SampleBuilder;
///
/// let builder = SampleBuilder::new();
/// let mut bots:Vec<_>= builder.build().take(1000).collect();
/// let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|a|builder.create_aabb(a)).build_seq();
/// //Use tree
/// ```
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
    
    ///Build in parallel
    #[inline(always)]
    pub fn build_par(&mut self) -> DinoTree<'a,A,Num,T> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut SplitterEmpty)
    }
}

impl<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeBuilder<'a, A, T, Num, F>
{
    ///Create a dinotree builder.
    ///The user picks the axis along which the first divider will partition.
    ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
    ///The user also passes a function to create the bounding box of each bot in the slice passed.
    #[inline(always)]
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

    ///Choose a custom bin stratagy.
    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    ///Choose a custom height for the tree.
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
    #[inline(always)]
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> DinoTree<'a,A,Num,T> {
        #[repr(transparent)]
        pub struct SplitterWrap<S> {
            inner: S,
        }
        impl<S: Splitter> Splitter for SplitterWrap<S> {
            #[inline(always)]
            fn div(&mut self) -> Self {
                SplitterWrap {
                    inner: self.inner.div(),
                }
            }
            #[inline(always)]
            fn add(&mut self, a: Self) {
                self.inner.add(a.inner)
            }
            #[inline(always)]
            fn node_start(&mut self) {
                self.inner.node_start();
            }
            #[inline(always)]
            fn node_end(&mut self) {
                self.inner.node_end()
            }
        }

        unsafe impl<S> Send for SplitterWrap<S> {}
        let splitter: &mut SplitterWrap<S> =
            unsafe { &mut *((splitter as *mut S) as *mut SplitterWrap<S>) };
        self.build_inner(par::Sequential, DefaultSorter, splitter)
    }

    ///Build sequentially.
    #[inline(always)]
    pub fn build_seq(&mut self) -> DinoTree<'a,A,Num,T> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut SplitterEmpty,
        )
    }


    pub(crate) fn build_inner<JJ: par::Joiner, S: Splitter + Send>(
        &mut self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut S,
    ) -> DinoTree<'a,A,Num,T> {
        
        let bots:&mut [T]=core::mem::replace::<&mut [T]>(&mut self.bots,&mut []);
        let axis = self.axis;
        let aabb_create = &mut self.aabb_create;

        let height = self.height;
        let binstrat = self.rebal_strat;

        let num_bots = bots.len();
        let max = core::u32::MAX;

        assert!(
            num_bots < max as usize,
            "problems of size {} are bigger are not supported",
            max
        );

        let mut conts: Vec<_> = bots
            .iter_mut()
            .map(move |k| unsafe{BBoxSendSync::new(aabb_create(k),k)})
            .collect();

        let (new_bots, new_tree) = {
            let cont_tree = create_tree(axis, par, &mut conts, sorter, ka, height, binstrat);

            let mut new_bots: Vec<_> = conts.drain(..)
                .map(|a| {
                    let (rect,inner)=a.into_inner();
                    BBoxMut::new(rect,inner)    
                })
                .collect();




            let new_nodes = {
                let mut rest: Option<&mut [BBoxMut<Num, _>]> = Some(&mut new_bots);
                let mut new_nodes = Vec::with_capacity(cont_tree.get_nodes().len());

                
                for node in cont_tree.get_nodes().iter() {
                    let (b, rest2) = rest.take().unwrap().split_at_mut(node.get().bots.len());
                    rest = Some(rest2);
                    let b = tools::Unique::new(ElemSlice::from_slice_mut(b) as *mut _).unwrap();
                    new_nodes.push(Node {
                        range: b,
                        cont: node.cont,
                        div: node.div,
                    })
                }
                new_nodes
            };

            (
                new_bots,
                compt::dfs_order::CompleteTreeContainer::from_preorder(new_nodes).unwrap(),
            )
        };

        DinoTree{
            inner:DinoTreeInner {
            axis,
            bots: new_bots,
            tree: new_tree,
            }
        }
    }
}

 

