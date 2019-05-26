use crate::copy::*;
use crate::inner_prelude::*;

pub struct NotSorted<A: AxisTrait, T: HasAabb>(pub DinoTree<A, T>);




///Builder for a DinoTree
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree::DinoTreeBuilder;
/// use dinotree_sample::SampleBuilder;
///
/// let builder = SampleBuilder::new();
/// let mut bots:Vec<_>= builder.build().take(1000).collect();
/// let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|a|builder.create_aabb(a)).build_seq();
/// //Use tree
/// ```
pub struct NotSortedBuilder<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>> {
	inner:DinoTreeBuilder<'a,A,T,Num,F>
}

impl<'a, A: AxisTrait, T: Copy, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    NotSortedBuilder<'a, A, T, Num, F>
{
    ///Create a dinotree builder.
    ///The user picks the axis along which the first divider will partition.
    ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
    ///The user also passes a function to create the bounding box of each bot in the slice passed.
    pub fn new(axis: A, bots: &[T], aabb_create: F) -> NotSortedBuilder<A, T, Num, F> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        let inner=DinoTreeBuilder {
            axis,
            bots,
            aabb_create,
            rebal_strat,
            height,
            height_switch_seq,
        };
        NotSortedBuilder{inner}
    }

    ///Choose a custom bin stratagy.
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.inner.rebal_strat = strat;
        self
    }

    ///Choose a custom height for the tree.
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.inner.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.inner.height_switch_seq = height;
        self
    }

    

    ///Build a not sorted dinotree with a splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> NotSorted<A, BBox<Num, T>> {
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
        NotSorted(self.inner.build_inner(par::Sequential, NoSorter, splitter))
    }


    ///Build not sorted sequentially
    pub fn build_seq(&mut self) -> NotSorted<A, BBox<Num, T>> {
        NotSorted(self.inner.build_inner(
            par::Sequential,
            NoSorter,
            &mut crate::advanced::SplitterEmpty,
        ))
    }

    ///Build not sorted in parallel
    pub fn build_par(&mut self) -> NotSorted<A, BBox<Num, T>> {
        let dlevel = compute_default_level_switch_sequential(self.inner.height_switch_seq, self.inner.height);
        NotSorted(self.inner.build_inner(dlevel, NoSorter, &mut crate::advanced::SplitterEmpty))
    }

}
