use crate::tree::*;
use crate::inner_prelude::*;



///A wrapper type where you are allowed to modify the aabb.
#[derive(Copy,Clone)]
#[repr(C)]
pub struct BBoxMut<N:NumTrait,T>{
    pub aabb:axgeom::Rect<N>,
    pub inner:T
}

impl<N:NumTrait,T> BBoxMut<N,T>{
    pub fn new(aabb:axgeom::Rect<N>,inner:T)->BBoxMut<N,T>{
        BBoxMut{aabb,inner}
    }
}


enum ReOrderStrat{
    Aux,
    NoAux
}


///Builder for a DinoTree
pub struct DinoTreeNoCopyBuilder<'a, A: AxisTrait, N:NumTrait,T:Copy> {
    axis: A,
    bots: &'a mut [BBox<N,T>],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}

impl<'a, A: AxisTrait, N:NumTrait,T:Copy> DinoTreeNoCopyBuilder<'a, A, N,T> {
    #[inline]
    pub fn new(axis: A, bots: &'a mut [BBoxMut<N,T>]) -> DinoTreeNoCopyBuilder<'a, A,N, T> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        let bots=unsafe{&mut *(bots as *mut [BBoxMut<N,T>] as *mut [BBox<N,T>])};
        DinoTreeNoCopyBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }


    #[inline]
    pub fn build_seq_aux(self)->DinoTreeNoCopy<'a,A,BBox<N,T>>{
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut crate::advanced::SplitterEmpty,
            ReOrderStrat::Aux
        )
    }

    #[inline]
    pub fn build_par_aux(self)->DinoTreeNoCopy<'a,A,BBox<N,T>>{
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut crate::advanced::SplitterEmpty,ReOrderStrat::Aux)
    }

    #[inline]
    pub fn build_seq(self) -> DinoTreeNoCopy<'a, A, BBox<N,T>> {
        self.build_inner(
            par::Sequential,
            DefaultSorter,
            &mut crate::advanced::SplitterEmpty,
            ReOrderStrat::NoAux
        )
    }

    

    #[inline]
    pub fn build_par(self) -> DinoTreeNoCopy<'a, A, BBox<N,T>> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        self.build_inner(dlevel, DefaultSorter, &mut crate::advanced::SplitterEmpty,ReOrderStrat::NoAux)
    }

    fn build_inner<JJ: par::Joiner, K: Splitter + Send>(
        mut self,
        par: JJ,
        sorter: impl Sorter,
        ka: &mut K,
        reorder_type:ReOrderStrat
    ) -> DinoTreeNoCopy<'a, A, BBox<N,T>> {
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
            .map(move |(index, k)| Cont2 {
                rect: *k.get(),
                index: index as u32,
            })
            .collect();

        let new_tree = {
            let mut cont_tree = ContTree::new(axis, par, &mut conts, sorter, ka, height, binstrat);

            {
                let mut indicies=reorder::swap_index(cont_tree.get_conts().iter().map(|a|a.index));
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

            compt::dfs_order::CompleteTreeContainer::from_preorder(new_nodes).unwrap()
        };
        let mover = conts
            .drain(..)
            .map(|a| a.index)
            .collect();

        DinoTreeNoCopy {
            mover,
            axis,
            bots: bots2,
            nodes: new_tree,
        }
    }
}


///Version of dinotree that does not make a copy of all the elements.
pub struct DinoTreeNoCopy<'a, A: AxisTrait, T: HasAabb> {
    axis: A,
    bots: &'a mut [T],
    nodes: compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>,
    mover: Vec<u32>,
}

impl<'a, A: AxisTrait, T: HasAabb + Copy> DinoTreeNoCopy<'a, A, T> {
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


impl<'a,A:AxisTrait,T:HasAabb> DinoTreeRefTrait for DinoTreeNoCopy<'a,A,T>{
    type Item=T;
    type Axis=A;
    type Num=T::Num;
    
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


impl<'a,A:AxisTrait,T:HasAabb> DinoTreeRefMutTrait for DinoTreeNoCopy<'a,A,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.nodes.vistr_mut(),
        }
    }
}
