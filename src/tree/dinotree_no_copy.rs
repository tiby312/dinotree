use super::*;

///Builder for a DinoTree
pub struct DinoTreeNoCopyBuilder<'a, A: AxisTrait, T: HasAabb + Copy> {
    axis: A,
    bots: &'a mut [T],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}
impl<'a, A: AxisTrait, T: HasAabb + Copy> DinoTreeNoCopyBuilder<'a, A, T> {
    #[inline]
    pub fn new(axis: A, bots: &'a mut [T]) -> DinoTreeNoCopyBuilder<'a, A, T> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeNoCopyBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }

    #[inline]
    pub fn build_seq(self) -> DinoTreeNoCopy<'a, A, T> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut crate::advanced::SplitterEmpty,
        )
    }

    #[inline]
    pub fn build_par(self) -> DinoTreeNoCopy<'a, A, T> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut crate::advanced::SplitterEmpty)
    }

    fn build_inner<JJ: par::Joiner, K: Splitter + Send>(
        mut self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut K,
    ) -> DinoTreeNoCopy<'a, A, T> {
        let axis = self.axis;

        let height = self.height;
        let binstrat = self.rebal_strat;

        let bots2 = unsafe { &mut *(self.bots as *mut [_]) };
        use crate::tree::cont_tree::*;

        let num_bots = self.bots.len();
        let max = std::u32::MAX;

        assert!(
            num_bots < max as usize,
            "problems of size {} are bigger are not supported",
            max
        );

        let mut conts: Vec<_> = self
            .bots
            .iter()
            .enumerate()
            .map(|(index, k)| Cont2 {
                rect: *k.get(),
                index: index as u32,
            })
            .collect();

        let new_tree = {
            let mut cont_tree = ContTree::new(axis, par, &mut conts, sorter, ka, height, binstrat);

            let new_bots = {
                impl<Num: NumTrait> reorder::HasIndex for Cont2<Num> {
                    fn get(&self) -> usize {
                        self.index as usize
                    }
                    fn set(&mut self, index: usize) {
                        self.index = index as u32;
                    }
                }
                //bots
                reorder::reorder(&mut self.bots, cont_tree.get_conts_mut())
            };

            let new_nodes = {
                let mut rest: Option<&mut [T]> = Some(new_bots);
                let mut new_nodes = Vec::with_capacity(cont_tree.get_tree().get_nodes().len());
                for node in cont_tree.get_tree_mut().get_nodes().iter() {
                    let (b, rest2) = rest.take().unwrap().split_at_mut(node.get().bots.len());
                    rest = Some(rest2);
                    let b = unsafe { std::ptr::Unique::new_unchecked(b as *mut [_]) };
                    new_nodes.push(Node {
                        range: b,
                        cont: node.cont,
                        div: node.div,
                    })
                }
                new_nodes
            };

            compt::dfs_order::CompleteTreeContainer::from_preorder(new_nodes).unwrap()
        };
        let mover = conts
            .drain(..)
            .map(|a| crate::tree::dinotree_no_copy::Index(a.index))
            .collect();

        DinoTreeNoCopy {
            mover,
            axis,
            bots: bots2,
            nodes: new_tree,
        }
    }
}

pub struct Index(pub u32);
impl reorder::HasIndex for Index {
    fn get(&self) -> usize {
        self.0 as usize
    }
    fn set(&mut self, index: usize) {
        self.0 = index as u32;
    }
}

///A version where the bots are not copied. This means that the slice borrowed from the user
///must remain borrowed for the entire lifetime of the tree.
pub struct DinoTreeNoCopy<'a, A: AxisTrait, T: HasAabb> {
    axis: A,
    bots: &'a mut [T],
    nodes: compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>,
    mover: Vec<Index>,
}

impl<'a, A: AxisTrait, T: HasAabb + Copy> DinoTreeNoCopy<'a, A, T> {
    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    #[inline]
    pub fn into_original(mut self) -> &'a mut [T] {
        reorder::reorder(self.bots, &mut self.mover)
    }

    ///Return a mutable reference to the tree.
    #[inline]
    pub fn as_ref_mut(&mut self) -> DinoTreeRefMut<A, T> {
        DinoTreeRefMut {
            axis: self.axis,
            bots: self.bots,
            tree: &mut self.nodes,
        }
    }

    ///Return a reference to the tree.
    #[inline]
    pub fn as_ref(&self) -> DinoTreeRef<A, T> {
        DinoTreeRef {
            axis: self.axis,
            bots: self.bots,
            tree: &self.nodes,
        }
    }
}
