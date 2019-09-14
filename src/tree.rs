
use crate::inner_prelude::*;
//pub use assert_invariants::assert_invariants;




pub mod dinotree_direct{
/*
    use crate::inner_prelude::*;
    pub struct DinotreeDirectBaggage<N,T>{
        inner:Vec<BBox<N,T>>
    }

    pub fn create_bbox_direct<Num:NumTrait,T>(bots:&[T],mut aabb_create:impl FnMut(&T)->Rect<Num>)->Vec<BBox<Num,u32>>{
        bots.iter().enumerate().map(|(index,bot)|BBox::new(aabb_create(bot),index as u32)).collect()
    }

    pub fn create_bbox_direct_tree<'a,A:AxisTrait,N:NumTrait,T:Copy>(bots:&'a mut [T],tree:&DinoTree<A,BBox<N,u32>>)->DinotreeDirectBaggage<N,T>
    {

        let rev:Vec<u32>=tree.inner.get_nodes().iter().flat_map(|a|a.range.as_ref().iter()).map(|a|a.inner).collect();


        let mut bots:Vec<BBox<N,T>>=tree.inner.get_nodes().iter().flat_map(|a|a.range.as_ref().iter()).map(|a|{
                BBox::new(a.rect,bots[a.inner as usize])
            }).collect();

        DinotreeDirectBaggage{inner:bots}
    }
    pub fn create_bbox_direct_tree2<'a,A:AxisTrait,N:NumTrait,T>(
        mut bots:&'a mut DinotreeDirectBaggage<N,T>,
        tree:DinoTree<A,BBox<N,u32>>)->DinoTree<'a,A,BBox<N,T>>{
        
        let mut nodes=Vec::new();

        let mut k=Some(&mut bots.inner as &mut [_]);
        for NodeMut{range,cont,div} in tree.inner.into_nodes().drain(..){
            
            let (first,mut rest) = k.take().unwrap().split_at_mut(range.len());
            nodes.push(NodeMut{range:ProtectedBBoxSlice::new(first),cont,div});
            k=Some(rest);
        }
        DinoTree{axis:tree.axis,inner:compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap()}
    }
*/
}


pub mod dinotree_indirect{
    use crate::inner_prelude::*;
    pub fn create_bbox_indirect<'a,T:HasAabb>(bots:&'a mut [T])->Vec<BBoxIndirect<'a,T>>
    {
        bots.iter_mut().map(|a|BBoxIndirect{inner:a}).collect()
    }    
}

pub mod dinotree_good{
    use crate::inner_prelude::*;

    pub fn create_bbox_mut<'a,Num:NumTrait,T>(bots:&'a mut [T],mut aabb_create:impl FnMut(&T)->Rect<Num>)->Vec<BBoxMut<'a,Num,T>>{
        bots
            .iter_mut()
            .map(move |k| BBoxMut::new(aabb_create(k),k))
            .collect()
    }    
}

pub mod dinotree_owned{
    use crate::inner_prelude::*;

  


}

pub mod notsorted{
    use crate::inner_prelude::*;    

    pub struct NotSorted<A: AxisTrait,N:NodeTrait>(DinoTree<A,N>);

    impl<A:AxisTrait,N:NodeTrait> NotSorted<A,N>{

        #[inline(always)]
        pub fn axis(&self)->A{
            self.0.axis()
        }

        #[inline(always)]
        pub fn vistr(&self)->Vistr<N>{
            Vistr {
                inner: self.0.inner.vistr(),
            }
        }

        #[inline(always)]
        pub fn vistr_mut(&mut self)->VistrMut<N>{
            VistrMut {
                inner: self.0.inner.vistr_mut(),
            }
        }

    }





    pub struct NotSortedBuilder<'a, A: AxisTrait, T:HasAabb, F: FnMut(&T) -> Rect<T::Num>> (
        DinoTreeBuilder<'a,A,T,F>
    );


    impl<'a, A: AxisTrait, T: HasAabb+Send+Sync, F: FnMut(&T) -> Rect<T::Num>>
        NotSortedBuilder<'a, A, T, F>
    {

        ///Build not sorted in parallel
        pub fn build_par(mut self) -> NotSorted<A,NodeMut<'a,T>> {
            let dlevel = compute_default_level_switch_sequential(self.0.height_switch_seq, self.0.height);
            let inner = create_tree_par(self.0.axis,dlevel, self.0.bots, NoSorter, &mut SplitterEmpty, self.0.height, self.0.rebal_strat);
            NotSorted(DinoTree{axis:self.0.axis,inner})
        }
    }
    impl<'a, A: AxisTrait, T:HasAabb, F: FnMut(&T) -> Rect<T::Num>>
        NotSortedBuilder<'a, A, T, F>
    {
        ///Create a dinotree builder.
        ///The user picks the axis along which the first divider will partition.
        ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
        ///The user also passes a function to create the bounding box of each bot in the slice passed.
        pub fn new(axis: A, bots: &'a mut [T], aabb_create: F) -> NotSortedBuilder<'a,A, T, F> {
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
            NotSortedBuilder(inner)
        }

        ///Choose a custom bin stratagy.
        pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
            self.0.rebal_strat = strat;
            self
        }

        ///Choose a custom height for the tree.
        pub fn with_height(&mut self, height: usize) -> &mut Self {
            self.0.height = height;
            self
            //TODO test corner cases of this
        }

        ///Choose the height at which to switch from parallel to sequential.
        ///If you end up building sequentially, this argument is ignored.
        pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
            self.0.height_switch_seq = height;
            self
        }

        

        ///Build a not sorted dinotree with a splitter.
        pub fn build_with_splitter_seq<S: Splitter>(
            mut self,
            splitter: &mut S,
        ) -> NotSorted<A,NodeMut<'a,T>> {
            let inner = create_tree_seq(self.0.axis, self.0.bots, NoSorter, splitter, self.0.height, self.0.rebal_strat);
            NotSorted(DinoTree{axis:self.0.axis,inner}) 
        }


        ///Build not sorted sequentially
        pub fn build_seq(mut self) -> NotSorted<A,NodeMut<'a,T>> {
            let inner = create_tree_seq(self.0.axis, self.0.bots, NoSorter, &mut SplitterEmpty, self.0.height, self.0.rebal_strat);
            NotSorted(DinoTree{axis:self.0.axis,inner})
        }


    }

}
pub struct DinoTree<A:AxisTrait,N:NodeTrait>{
    axis:A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>,
}
impl<A:AxisTrait,N:NodeTrait> DinoTree<A,N>{
    pub fn axis(&self)->A{
        self.axis
    }
    pub fn vistr_mut(&mut self)->VistrMut<N>{
        VistrMut{inner:self.inner.vistr_mut()}
    }
    pub fn vistr(&self)->Vistr<N>{
        Vistr{inner:self.inner.vistr()}
    }
    pub fn get_height(&self)->usize{
        self.inner.get_height()
    }
}


pub struct DinoTreeBuilder<'a, A: AxisTrait, T, F> {
    pub(crate) axis: A,
    pub(crate) bots: &'a mut [T],
    pub(crate) aabb_create: F,
    pub(crate) rebal_strat: BinStrat,
    pub(crate) height: usize,
    pub(crate) height_switch_seq: usize,
}


impl<'a,A: AxisTrait, T:HasAabb+Send+Sync, F: FnMut(&T) -> Rect<T::Num>>
    DinoTreeBuilder<'a,A,  T, F>
{
    pub fn build_par(self) -> DinoTree<A,NodeMut<'a,T>> {
        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);
        let inner = create_tree_par(self.axis,dlevel, self.bots, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
        DinoTree{axis:self.axis,inner}
    }
}

impl<'a, A: AxisTrait, T:HasAabb,  F: FnMut(&T) -> Rect<T::Num>> DinoTreeBuilder<'a,A,T,F>{
    pub fn new(axis: A, bots: &mut [T], aabb_create: F) -> DinoTreeBuilder<A, T, F> {
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

    pub fn build_seq(self)->DinoTree<A,NodeMut<'a,T>>{
        let inner = create_tree_seq(self.axis, self.bots, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
        DinoTree{axis:self.axis,inner}
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
    ) -> DinoTree<A,NodeMut<'a,T>> {
        let inner = create_tree_seq(self.axis, self.bots, DefaultSorter, splitter, self.height, self.rebal_strat);
        DinoTree{axis:self.axis,inner} 
    }


}


/*
pub(crate) struct DinoTreeInner<A: AxisTrait, T:HasAabb> {
    pub axis: A,
    pub bots: Vec<T>,
    pub tree: compt::dfs_order::CompleteTreeContainer<NodeMut<'aT>, compt::dfs_order::PreOrder>,
}
*/



///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);

    ///Called at the start of the recursive call.
    fn node_start(&mut self);

    ///It is important to note that this gets called in other places besides in the final recursive call of a leaf.
    ///They may get called in a non leaf if its found that there is no more need to recurse further.
    fn node_end(&mut self);
}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty {
    fn div(&mut self) -> Self {
        SplitterEmpty
    }
    fn add(&mut self, _: Self) {}
    fn node_start(&mut self) {}
    fn node_end(&mut self) {}
}


/*
///The trait through which dinotree algorithms may use the tree.
///We expose a trait so that both the copy and non copy version of the tree have
///the same interface.
pub trait DinoTreeRefTrait<'b>{
    type Item:HasAabb<Num=Self::Num>;
    type Axis:AxisTrait;
    type Num:NumTrait;
    //type Inner;

    fn axis(&self)->Self::Axis;
    

    //fn vistr(&self)->Vistr<Self::Item>;
    fn vistr<'a>(&'a self)->compt::dfs_order::Vistr<'a, NodeMut<'b,Self::Item>, compt::dfs_order::PreOrder>;

    ///Return the height of the dinotree.
    fn height(&self) -> usize;

    ///Return the number of nodes of the dinotree.
    fn num_nodes(&self) -> usize;

    ///Return the number of bots in the tree.
    fn num_bots(&self) -> usize;

}

///The mutable part of the tree accessing trait.
pub trait DinoTreeRefMutTrait<'b>:DinoTreeRefTrait<'b>{
//    fn vistr_mut(&mut self)->VistrMut<Self::Item>;
      fn vistr_mut<'a>(&'a mut self)->compt::dfs_order::VistrMut<'a, NodeMut<'b,Self::Item>, compt::dfs_order::PreOrder>;

}
*/



///Outputs the height given an desirned number of bots per node.
#[inline]
pub fn compute_tree_height_heuristic_debug(num_bots: usize, num_per_node: usize) -> usize {
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        let a=num_bots as f32 / num_per_node as f32;
        let b=a.log2()/2.0;
        let c=(b.ceil() as usize)*2+1;
        c
    }
}

///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn default_level_switch_sequential() -> usize {
    const DEPTH_SEQ: usize = 6;
    DEPTH_SEQ
}

///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn compute_default_level_switch_sequential(depth: usize, height: usize) -> par::Parallel {
    //const DEPTH_SEQ:usize=4;
    let dd = depth;

    let gg = if height <= dd { 0 } else { height - dd };

    par::Parallel::new(Depth(gg))
}

///Returns the height of a dyn tree for a given number of bots.
///The height is chosen such that the nodes will each have a small amount of bots.
///If we had a node per bot, the tree would be too big.
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///This is provided so that users can allocate enough space for all the nodes
///before the tree is constructed, perhaps for some graphics buffer.
#[inline]
pub fn compute_tree_height_heuristic(num_bots: usize) -> usize {
    //we want each node to have space for around num_per_node bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    //Make this number too small, and the tree will have too many levels,
    //and too much time will be spent recursing.
    //Make this number too high, and you will lose the properties of a tree,
    //and you will end up with just sweep and prune.
    //This number was chosen emprically from running the dinotree_alg_data project,
    //on two different machines.
    compute_tree_height_heuristic_debug(num_bots,128)
}



/// Tree Iterator that returns a reference to each node.
pub struct Vistr<'a, N:NodeTrait> {
    pub(crate) inner: compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>,
}

impl<'a, N:NodeTrait> Vistr<'a, N> {
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    #[inline]
    pub fn create_wrap(&self) -> Vistr<N> {
        Vistr {
            inner: self.inner.create_wrap(),
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        //Safe since we know Vistr implements FixedDepthVisitor.
        self.inner.level_remaining_hint().0
    }

}

unsafe impl<'a, N:NodeTrait> compt::FixedDepthVisitor for Vistr<'a, N> {}

impl<'a, N:NodeTrait> Visitor for Vistr<'a, N> {
    type Item = &'a N;

    #[inline(always)]
    fn next(self) -> (Self::Item, Option<[Self; 2]>) {
        let (nn, rest) = self.inner.next();

        let k = match rest {
            Some([left, right]) => Some([Vistr { inner: left }, Vistr { inner: right }]),
            None => None,
        };
        (nn, k)
    }
    
    #[inline(always)]
    fn level_remaining_hint(&self) -> (usize, Option<usize>) {
        self.inner.level_remaining_hint()
    }

    
    #[inline(always)]
    fn dfs_preorder(self,mut func:impl FnMut(Self::Item)){
        self.inner.dfs_preorder(|a|{
            func(a)
        });
    }
}


/// Tree Iterator that returns a mutable reference to each node.
pub struct VistrMut<'a, N:NodeTrait> {
    pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
}

impl<'a,N:NodeTrait> VistrMut<'a, N> {
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    
    #[inline(always)]
    pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
        VistrMut {
            inner: self.inner.create_wrap_mut(),
        }
    }

    
    #[inline(always)]
    pub fn height(&self) -> usize {
        //Safe since we know Vistr implements FixedDepthVisitor.
        self.inner.level_remaining_hint().0
    }



}


impl<'a,N:NodeTrait> core::ops::Deref for VistrMut<'a,N> {
    type Target = Vistr<'a, N>;
    
    #[inline(always)]
    fn deref(&self) -> &Vistr<'a, N> {
        unsafe { &*(self as *const VistrMut<N> as *const Vistr<N>) }
    }
}



unsafe impl<'a, N:NodeTrait> compt::FixedDepthVisitor for VistrMut<'a, N> {}

impl<'a,N:NodeTrait> Visitor for VistrMut<'a, N> {
    type Item = &'a mut N;

    
    #[inline(always)]
    fn next(self) -> (Self::Item, Option<[Self; 2]>) {
        let (nn, rest) = self.inner.next();

        let k = match rest {
            Some([left, right]) => Some([VistrMut { inner: left }, VistrMut { inner: right }]),
            None => None,
        };
        (nn, k)
    }
    
    #[inline(always)]
    fn level_remaining_hint(&self) -> (usize, Option<usize>) {
        self.inner.level_remaining_hint()
    }


    
    #[inline(always)]
    fn dfs_preorder(self,mut func:impl FnMut(Self::Item)){
        self.inner.dfs_preorder(|a|{
            func(a)
        });
    }
}





pub trait NodeTrait{
    type T:HasAabb<Num=Self::Num>;
    type Num:NumTrait;
    fn get(&self)->NodeRef<Self::T>;
    fn get_mut(&mut self)->NodeRefMut<Self::T>;
}

impl<'a,T:HasAabb> NodeTrait for NodeMut<'a,T>{
    type T=T;
    type Num=T::Num;
    fn get(&self)->NodeRef<Self::T>{
        NodeRef{bots:self.range.as_ref(),cont:&self.cont,div:&self.div}
    }
    fn get_mut(&mut self)->NodeRefMut<Self::T>{
        NodeRefMut{bots:self.range.as_mut(),cont:&self.cont,div:&self.div}
    }
}

///A node in a dinotree.
pub struct NodeMut<'a,T: HasAabb> {
    pub range: ProtectedBBoxSlice<'a,T>,

    //range is empty iff cont is none.
    pub cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    pub div: Option<T::Num>,
}



///Mutable reference to a node in the dinotree.
pub struct NodeRefMut<'a, T:HasAabb> {
    ///The bots that belong to this node.
    pub bots: ProtectedBBoxSlice<'a,T>,

    ///Is None iff bots is empty.
    pub cont: &'a Option<axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: &'a Option<T::Num>,
}


///Reference to a node in the dinotree.
pub struct NodeRef<'a, T:HasAabb> {
    ///The bots that belong to this node.
    pub bots: &'a [T],

    ///Is None iff bots is empty.
    pub cont: &'a Option<axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: &'a Option<T::Num>,
}


/*

///A node in a dinotree.
pub(crate) struct Node<T: HasAabb> {
    pub(crate) range: tools::Unique<[T]>,

    //range is empty iff cont is none.
    pub(crate) cont: axgeom::Range<T::Num>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    pub(crate) div: Option<T::Num>,
}





impl<T:HasAabb> Node<T>{

    #[inline(always)]
    pub fn get(&self) -> NodeRef<T> {
        let bots = unsafe { &*self.range.as_ptr() };
        let cont = if bots.is_empty() {
            None
        } else {
            Some(&self.cont)
        };

        NodeRef {
            bots,
            cont,
            div: self.div.as_ref(),
        }
    }

}
impl<T:HasAabb> Node<T> {

    
    
    #[inline(always)]
    pub fn get_mut(&mut self) -> NodeRefMut<T> {
        let bots = unsafe { &mut *self.range.as_ptr() };
        let cont = if bots.is_empty() {
            None
        } else {
            Some(&self.cont)
        };

        NodeRefMut {
            bots:ProtectedBBoxSlice::new(bots),
            cont,
            div: self.div.as_ref(),
        }
    }
    
}
*/



pub(crate) trait Sorter: Copy + Clone + Send + Sync {
    fn sort(&self, axis: impl AxisTrait, bots: &mut [impl HasAabb]);
}

#[derive(Copy, Clone)]
pub(crate) struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl AxisTrait, bots: &mut [impl HasAabb]) {
        oned::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone)]
pub(crate) struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl AxisTrait, _bots: &mut [impl HasAabb]) {}
}

fn nodes_left(depth: usize, height: usize) -> usize {
    let levels = height - depth;
    2usize.rotate_left(levels as u32) - 1
}



pub(crate) use self::cont_tree::create_tree_seq;
pub(crate) use self::cont_tree::create_tree_par;
mod cont_tree {

    use super::*;


    pub(crate) fn create_tree_seq<'a,A:AxisTrait,T:HasAabb,K:Splitter>(
            div_axis: A,
            rest: &'a mut [T],
            sorter: impl Sorter,
            splitter: &mut K,
            height: usize,
            binstrat: BinStrat,
            ) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a,T>, compt::dfs_order::PreOrder>{
        let num_bots = rest.len();
        
        let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

        let r = Recurser {
            height,
            binstrat,
            sorter,
            _p: PhantomData,
        };
        r.recurse_preorder_seq(div_axis, rest, &mut nodes, splitter, 0);

        let tree =
            compt::dfs_order::CompleteTreeContainer::from_preorder(
                nodes
            )
            .unwrap();


        let k = tree
            .get_nodes()
            .iter()
            .fold(0, |acc, a| acc + a.range.len());
        debug_assert_eq!(k, num_bots);

        tree
    }
    pub(crate) fn create_tree_par<'a,A:AxisTrait,JJ:par::Joiner,T:HasAabb+Send+Sync,K:Splitter+Send+Sync>(
            div_axis: A,
            dlevel: JJ,
            rest: &'a mut [T],
            sorter: impl Sorter,
            splitter: &mut K,
            height: usize,
            binstrat: BinStrat,
            ) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a,T>, compt::dfs_order::PreOrder>{
        let num_bots = rest.len();
        
        let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

        let r = Recurser {
            height,
            binstrat,
            sorter,
            _p: PhantomData,
        };
        r.recurse_preorder(div_axis, dlevel, rest, &mut nodes, splitter, 0);

        let tree =
            compt::dfs_order::CompleteTreeContainer::from_preorder(
                nodes
            )
            .unwrap();


        let k = tree
            .get_nodes()
            .iter()
            .fold(0, |acc, a| acc + a.range.len());
        debug_assert_eq!(k, num_bots);

        tree
    }

    struct Recurser<'a, T: HasAabb, K: Splitter, S: Sorter> {
        height: usize,
        binstrat: BinStrat,
        sorter: S,
        _p :PhantomData<(K,&'a T)>
    }


    impl<'a, T: HasAabb, K: Splitter , S: Sorter> Recurser<'a, T, K, S> {

        fn create_leaf<A:AxisTrait>(&self,axis:A,rest:&'a mut [T]) -> NodeMut<'a,T>{
            self.sorter.sort(axis.next(),rest);
                    
            let cont = create_cont(axis,rest);

            NodeMut {
                //range: unsafe{tools::Unique::new_unchecked(rest as *mut _)},
                range:ProtectedBBoxSlice::new(rest),
                cont,
                div: None,
            }
        }

        fn create_non_leaf<A:AxisTrait>(&self,axis:A,rest:&'a mut [T]) -> (NodeMut<'a,T>,&'a mut [T],&'a mut [T]){
            match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
                ConstructResult::NonEmpty {
                    cont,
                    div,
                    mid,
                    left,
                    right,
                } => (
                    NodeMut {
                        range: ProtectedBBoxSlice::new(mid),
                        cont,
                        div: Some(div),
                    },
                    left,
                    right,
                ),
                ConstructResult::Empty(empty) => {

                    let (a,empty) = tools::duplicate_empty_slice(empty);
                    let (b,c) = tools::duplicate_empty_slice(empty);
                        
                    let node = NodeMut {
                        range: ProtectedBBoxSlice::new(a),
                        cont:unsafe{core::mem::MaybeUninit::zeroed().assume_init()},
                        div: None,
                    };

                    (node,b,c)
                    /*
                    for _ in 0..compt::compute_num_nodes(self.height - depth) {
                        let a = tools::duplicate_empty_slice(empty);
                        let cont = unsafe { core::mem::uninitialized() };
                        let node = Node {
                            range: tools::Unique::new(ElemSlice::from_slice_mut(a) as *mut _).unwrap(),
                            cont,
                            div: None,
                        };
                        nodes.push(node);
                    }
                    return;
                    */
                }
            }
        }

        fn recurse_preorder_seq<A:AxisTrait>(
            &self,
            axis: A,
            rest: &'a mut [T],
            nodes: &mut Vec<NodeMut<'a,T>>,
            splitter: &mut K,
            depth: usize,
            )
        {
            splitter.node_start();

            if depth < self.height - 1 {
                let (node, left, right) = self.create_non_leaf(axis,rest); 
                nodes.push(node);

                let mut splitter2 = splitter.div();

                self.recurse_preorder_seq(
                    axis.next(),
                    left,
                    nodes,
                    splitter,
                    depth + 1,
                );
                self.recurse_preorder_seq(
                    axis.next(),
                    right,
                    nodes,
                    &mut splitter2,
                    depth + 1,
                );
                
                splitter.add(splitter2);
            } else {
                let node = self.create_leaf(axis,rest);
                nodes.push(node);
                splitter.node_end();
            }
        }
    }
    impl<'a, T: HasAabb + Send + Sync, K: Splitter + Send+ Sync , S: Sorter> Recurser<'a, T, K, S> {



        fn recurse_preorder<A: AxisTrait, JJ: par::Joiner>(
            &self,
            axis: A,
            dlevel: JJ,
            rest: &'a mut [T],
            nodes: &mut Vec<NodeMut<'a,T>>,
            splitter: &mut K,
            depth: usize,
        ) {
            splitter.node_start();

            if depth < self.height - 1 {
                let (node, left, right) = self.create_non_leaf(axis,rest);
                    
                nodes.push(node);

                let mut splitter2 = splitter.div();

                let splitter = match dlevel.next(Depth(depth)){
                    par::ParResult::Parallel([dleft,dright])=>{
                        let splitter2 = &mut splitter2;

                        let ((splitter, nodes), mut nodes2) = rayon::join(
                            move || {
                                self.recurse_preorder(
                                    axis.next(),
                                    dleft,
                                    left,
                                    nodes,
                                    splitter,
                                    depth + 1,
                                );
                                (splitter, nodes)
                            },
                            move || {
                                let mut nodes2: Vec<_> =
                                    Vec::with_capacity(nodes_left(depth, self.height));
                                self.recurse_preorder(
                                    axis.next(),
                                    dright,
                                    right,
                                    &mut nodes2,
                                    splitter2,
                                    depth + 1,
                                );
                                nodes2
                            },
                        );

                        nodes.append(&mut nodes2);
                        splitter
                    },
                    par::ParResult::Sequential(_)=>{
                        self.recurse_preorder_seq(
                            axis.next(),
                            left,
                            nodes,
                            splitter,
                            depth + 1,
                        );
                        self.recurse_preorder_seq(
                            axis.next(),
                            right,
                            nodes,
                            &mut splitter2,
                            depth + 1,
                        );
                        splitter
                    }
                };

                splitter.add(splitter2);
            } else {
                let node = self.create_leaf(axis,rest);
                nodes.push(node);
                splitter.node_end();
            }
        }
    }
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont(b: &mut test::Bencher) {
    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont2(b: &mut test::Bencher) {
    fn create_cont2<A: AxisTrait, T: HasAabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
        let left = middle
            .iter()
            .map(|a| a.get().get_range(axis).left)
            .min()
            .unwrap();
        let right = middle
            .iter()
            .map(|a| a.get().get_range(axis).right)
            .max()
            .unwrap();
        axgeom::Range { left, right }
    }

    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(|pos|  BBox::new(aabb_create_isize(pos, 5), ()) )
        .collect();

    b.iter(|| {
        let k = create_cont2(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

fn create_cont<A: AxisTrait, T: HasAabb>(axis: A, middle: &[T]) -> Option<axgeom::Range<T::Num>> {
    match middle.split_first(){
        Some((first,rest))=>{
            let mut min = first.get().get_range(axis).left;
            let mut max = first.get().get_range(axis).right;

            for a in rest.iter() {
                let left = &a.get().get_range(axis).left;
                let right = &a.get().get_range(axis).right;

                if *left < min {
                    min = *left;
                }

                if *right > max {
                    max = *right;
                }
            }

            Some(axgeom::Range {
                left: min,
                right: max,
            })
        },
        None=>{
            //We use unsafe here since we don't want to add more type contraints
            //on NumTrait. If we add Default, then it becomes hard to implement
            //some number types that count the number of comparisions made.

            //It is safe to do, since this cont will never be accessed.
            //Before we return a NodeRef, or NodeRefMut, we check if there are
            //any bots in the node, and only then return cont.
            //unsafe{core::mem::MaybeUninit::zeroed().assume_init()}
            None
        }
    }
    

}

///Passed to the binning algorithm to determine
///if the binning algorithm should check for index out of bounds.
#[derive(Copy, Clone, Debug)]
pub enum BinStrat {
    Checked,
    NotChecked,
}

enum ConstructResult<'a, T: HasAabb> {
    NonEmpty {
        div: T::Num,
        cont: Option<axgeom::Range<T::Num>>,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: HasAabb>(
    bin_strat: BinStrat,
    sorter: impl Sorter,
    div_axis: impl AxisTrait,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure = |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

        let k = {
            let mm = bots.len() / 2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).left
    };

    //TODO. its possible that middle is empty is the ranges inserted had
    //zero length.
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned = match bin_strat {
        BinStrat::Checked => oned::bin_middle_left_right(div_axis, &med, bots),
        BinStrat::NotChecked => unsafe {
            oned::bin_middle_left_right_unchecked(div_axis, &med, bots)
        },
    };

    //debug_assert!(!binned.middle.is_empty());
    sorter.sort(div_axis.next(), binned.middle);
            
    let cont = create_cont(div_axis, binned.middle);

    //We already know that the middile is non zero in length.
    
    ConstructResult::NonEmpty {
        mid: binned.middle,
        cont,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
