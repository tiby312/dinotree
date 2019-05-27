

use crate::tree::*;

use super::notsorted::*;
use crate::inner_prelude::*;

///Version of dinotree that makes a copy of all the elements.
pub struct DinoTree<A: AxisTrait, T: HasAabb> {
    pub(crate) axis: A,
    pub(crate) bots: Vec<T>,
    pub(crate) tree: compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>,
    pub(crate) mover: Vec<u32>,
}

impl<A:AxisTrait,T:HasAabb> DinoTree<A,T>{
    pub fn get_bots_mut(&mut self)->&mut [T]{
        &mut self.bots
    }
    pub fn get_bots(&self)->&[T]{
        &self.bots
    }
}

impl<A:AxisTrait,T:HasAabb> DinoTreeRefTrait for DinoTree<A,T>{
    type Item=T;
    type Axis=A;
    type Num=T::Num;
    
    fn axis(&self)->Self::Axis{
        self.axis
    }
    fn vistr(&self)->Vistr<Self::Item>{
        Vistr {
            inner: self.tree.vistr(),
        }
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        self.tree.get_height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        self.tree.get_nodes().len()
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        self.bots.len()
    }

}


impl<A:AxisTrait,T:HasAabb> DinoTreeRefMutTrait for DinoTree<A,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.tree.vistr_mut(),
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
    pub(crate) bots: &'a [T],
    pub(crate) aabb_create: F,
    pub(crate) rebal_strat: BinStrat,
    pub(crate) height: usize,
    pub(crate) height_switch_seq: usize,
}

impl<'a, A: AxisTrait, T: Copy, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeBuilder<'a, A, T, Num, F>
{
    ///Create a dinotree builder.
    ///The user picks the axis along which the first divider will partition.
    ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
    ///The user also passes a function to create the bounding box of each bot in the slice passed.
    pub fn new(axis: A, bots: &[T], aabb_create: F) -> DinoTreeBuilder<A, T, Num, F> {
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
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    ///Choose a custom height for the tree.
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.height_switch_seq = height;
        self
    }

    ///Build with a Splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> DinoTree<A, BBox<Num, T>> {
        #[repr(transparent)]
        pub struct SplitterWrap<S> {
            inner: S,
        }
        impl<S: Splitter> Splitter for SplitterWrap<S> {
            fn div(&mut self) -> Self {
                SplitterWrap {
                    inner: self.inner.div(),
                }
            }
            fn add(&mut self, a: Self) {
                self.inner.add(a.inner)
            }
            fn node_start(&mut self) {
                self.inner.node_start();
            }
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
    pub fn build_seq(&mut self) -> DinoTree<A, BBox<Num, T>> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut crate::advanced::SplitterEmpty,
        )
    }

    ///Build in parallel
    pub fn build_par(&mut self) -> DinoTree<A, BBox<Num, T>> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut crate::advanced::SplitterEmpty)
    }

    pub(crate) fn build_inner<JJ: par::Joiner, S: Splitter + Send>(
        &mut self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut S,
    ) -> DinoTree<A, BBox<Num, T>> {
        
        let bots = self.bots;
        let axis = self.axis;
        let aabb_create = &mut self.aabb_create;

        let height = self.height;
        let binstrat = self.rebal_strat;

        let num_bots = bots.len();
        let max = std::u32::MAX;

        assert!(
            num_bots < max as usize,
            "problems of size {} are bigger are not supported",
            max
        );

        let mut conts: Vec<_> = bots
            .iter()
            .enumerate()
            .map(move |(index, k)| Cont2 {
                rect: aabb_create(k),
                index: index as u32,
            })
            .collect();

        let (new_bots, new_tree) = {
            let mut cont_tree = ContTree::new(axis, par, &mut conts, sorter, ka, height, binstrat);

            let mut new_bots: Vec<_> = cont_tree
                .get_conts()
                .iter()
                .map(|a| BBox {
                    rect: a.rect,
                    inner: *unsafe { bots.get_unchecked(a.index as usize) },
                })
                .collect();




            let new_nodes = {
                let mut rest: Option<&mut [BBox<Num, T>]> = Some(&mut new_bots);
                let mut new_nodes = Vec::with_capacity(cont_tree.get_tree().get_nodes().len());

                
                for node in cont_tree.get_tree_mut().get_nodes().iter() {
                    let (b, rest2) = rest.take().unwrap().split_at_mut(node.get().bots.len());
                    rest = Some(rest2);
                    let b = tools::Unique::new(b as *mut [_]).unwrap();
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

        let mover = conts.drain(..).map(|a| a.index).collect();

        DinoTree {
            mover,
            axis,
            bots: new_bots,
            tree: new_tree,
        }
    }
}

impl<A: AxisTrait, T: HasAabb> DinoTree<A, T> {
 
    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    pub fn apply<X>(&self, bots: &mut [X], conv: impl Fn(&T, &mut X)) {
        assert_eq!(bots.len(), self.bots.len());
        for (bot, mov) in self.bots.iter().zip_eq(self.mover.iter()) {
            let target = unsafe { bots.get_unchecked_mut(*mov as usize) };
            conv(bot, target);
        }
    }

    ///Apply changes to the bots in the tree (not the aabb) without recreating the tree.
    pub fn apply_into<X>(&mut self, bots: &[X], conv: impl Fn(&X, &mut T)) {
        assert_eq!(bots.len(), self.bots.len());

        let treev = self.bots.iter_mut();

        for (bot, mov) in treev.zip_eq(self.mover.iter()) {
            let source = unsafe { bots.get_unchecked(*mov as usize) };
            conv(source, bot)
        }
    }
}
