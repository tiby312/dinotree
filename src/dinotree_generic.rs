


use crate::tree::*;
use crate::inner_prelude::*;






enum ReOrderStrat{
    Aux,
    NoAux
}


///Builder for a DinoTree
pub struct DinoTreeGenericBuilder<'a, A: AxisTrait, T:HasAabb> {
    axis: A,
    bots: &'a mut [T],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}

impl<'a, A: AxisTrait, T:HasAabbMut> DinoTreeGenericBuilder<'a, A, T> {
    #[inline]
    pub fn new(axis: A, bots: &'a mut [T]) -> DinoTreeGenericBuilder<'a, A,T> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        let bots=unsafe{&mut *(bots as *mut [T] as *mut [T])};
        DinoTreeGenericBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }


    #[inline]
    pub fn build_seq_aux(self)->DinoTreeGeneric<'a,A,T>{
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut SplitterEmpty,
            ReOrderStrat::Aux
        )
    }

    #[inline]
    pub fn build_par_aux(self)->DinoTreeGeneric<'a,A,T>{
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut SplitterEmpty,ReOrderStrat::Aux)
    }

    #[inline]
    pub fn build_seq(self) -> DinoTreeGeneric<'a, A, T> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut SplitterEmpty,
            ReOrderStrat::NoAux
        )
    }

    

    #[inline]
    pub fn build_par(self) -> DinoTreeGeneric<'a, A, T> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut SplitterEmpty,ReOrderStrat::NoAux)
    }

    fn build_inner<JJ: par::Joiner, K: Splitter + Send>(
        mut self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut K,
        reorder_type:ReOrderStrat
    ) -> DinoTreeGeneric<'a, A, T> {
        let axis = self.axis;

        let height = self.height;
        let binstrat = self.rebal_strat;

        let bots2 = unsafe { &mut *(self.bots as *mut [_]) };
        
        let num_bots = self.bots.len();
        let max = core::u32::MAX;

        assert!(
            num_bots < max as usize,
            "problems of size {} are bigger are not supported",
            max
        );

        let mut conts: Vec<_> = self
            .bots
            .iter()
            .enumerate()
            .map(move |(index, k)| unsafe{BBoxSendSync::new(*k.get().rect,index as u32)})
            .collect();

        let new_tree = {
            let cont_tree = create_tree(axis, par, &mut conts, sorter, ka, height, binstrat);

            {
                let mut indicies=reorder::swap_index(conts.iter().map(|a|a.inner));
                match reorder_type{
                    ReOrderStrat::Aux=>{
                        reorder::reorder_index_aux(&mut self.bots, &mut indicies);
                    },
                    ReOrderStrat::NoAux=>{
                        reorder::reorder_index(&mut self.bots, &mut indicies);        
                    }
                }
                
            }    

            let new_nodes = {
                let mut rest: Option<&mut [_]> = Some(&mut self.bots);
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

            compt::dfs_order::CompleteTreeContainer::from_preorder(new_nodes).unwrap()
        };
        let mover = conts
            .drain(..)
            .map(|a| a.inner)
            .collect();

        DinoTreeGeneric {
            mover,
            axis,
            bots: bots2,
            nodes: new_tree,
        }
    }
}


///Version of dinotree that does not make a copy of all the elements.
pub struct DinoTreeGeneric<'a, A: AxisTrait, T: HasAabbMut> {
    axis: A,
    bots: &'a mut [T],
    nodes: compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>,
    mover: Vec<u32>,
}

impl<'a, A: AxisTrait, T: HasAabbMut> DinoTreeGeneric<'a, A, T> {
    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    #[inline]
    pub fn into_original(mut self) -> &'a mut [T] {
        reorder::reorder_index(self.bots, &mut self.mover);
        self.bots
    }

    ///Returns the elements of the tree is the order they are in the tree.
    pub fn get_bots_mut(&mut self)->&mut [T]{
        self.bots
    }

    pub fn get_bots(&self)->&[T]{
        self.bots
    }
}


impl<'a,A:AxisTrait,T:HasAabbMut> DinoTreeRefTrait for DinoTreeGeneric<'a,A,T>{
    type Item=T;
    type Axis=A;
    type Num=T::Num;
    type Inner=T::Inner;
    
    fn axis(&self)->Self::Axis{
        self.axis
    }
    fn vistr(&self)->Vistr<Self::Item>{
        Vistr {
            inner: self.nodes.vistr(),
        }
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        self.nodes.get_height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        self.nodes.get_nodes().len()
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        self.bots.len()
    }

}


impl<'a,A:AxisTrait,T:HasAabbMut> DinoTreeRefMutTrait for DinoTreeGeneric<'a,A,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.nodes.vistr_mut(),
        }
    }
}
