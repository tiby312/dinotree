
use crate::inner_prelude::*;
pub use assert_invariants::assert_invariants;

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



///The trait through which dinotree algorithms may use the tree.
///We expose a trait so that both the copy and non copy version of the tree have
///the same interface.
pub trait DinoTreeRefTrait where Self::Item:HasAabb<Num=Self::Num>{
    type Item:HasAabbMut<Inner=Self::Inner,Num=Self::Num>;
    type Axis:AxisTrait;
    type Num:NumTrait;
    type Inner;

    fn axis(&self)->Self::Axis;
    

    fn vistr(&self)->Vistr<Self::Item>;


    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize;

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize;

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize;

}

///The mutable part of the tree accessing trait.
pub trait DinoTreeRefMutTrait:DinoTreeRefTrait{
    fn vistr_mut(&mut self)->VistrMut<Self::Item>;
}



/*

impl<K:DinoTreeRefTrait> DinoTreeRefTrait for &K{
    type Item=K::Item;
    type Axis=K::Axis;
    type Num=K::Num;
    type Inner=K::Inner;
    
    fn axis(&self)->Self::Axis{
        K::axis(self)
    }
    fn vistr(&self)->Vistr<Self::Item>{
        K::vistr(self)
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        K::height(self)
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        K::num_nodes(self)
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        K::num_bots(self)
    }

}


impl<K:DinoTreeRefTrait> DinoTreeRefTrait for &mut K{
    type Item=K::Item;
    type Axis=K::Axis;
    type Num=K::Num;
    type Inner=K::Inner;
    
    fn axis(&self)->Self::Axis{
        K::axis(self)
    }
    fn vistr(&self)->Vistr<Self::Item>{
        K::vistr(self)
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        K::height(self)
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        K::num_nodes(self)
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        K::num_bots(self)
    }

}


impl<K:DinoTreeRefMutTrait> DinoTreeRefMutTrait for &mut K{  

    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        K::vistr_mut(self)
    }
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
pub struct Vistr<'a, T: HasAabb> {
    pub(crate) inner: compt::dfs_order::Vistr<'a, Node<T>, compt::dfs_order::PreOrder>,
}

impl<'a, T: HasAabbMut> Vistr<'a, T> {
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    #[inline]
    pub fn create_wrap(&self) -> Vistr<T> {
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

unsafe impl<'a, T: HasAabbMut> compt::FixedDepthVisitor for Vistr<'a, T> {}

impl<'a, T: HasAabbMut + 'a> Visitor for Vistr<'a, T> {
    type Item = NodeRef<'a, T>;

    #[inline]
    fn next(self) -> (Self::Item, Option<[Self; 2]>) {
        let (nn, rest) = self.inner.next();

        let k = match rest {
            Some([left, right]) => Some([Vistr { inner: left }, Vistr { inner: right }]),
            None => None,
        };
        (nn.get(), k)
    }
    #[inline]
    fn level_remaining_hint(&self) -> (usize, Option<usize>) {
        self.inner.level_remaining_hint()
    }

    #[inline]
    fn dfs_preorder(self,mut func:impl FnMut(Self::Item)){
        self.inner.dfs_preorder(|a|{
            func(a.get())
        });
    }
}


/// Tree Iterator that returns a mutable reference to each node.
pub struct VistrMut<'a, T:HasAabbMut> {
    pub(crate) inner: compt::dfs_order::VistrMut<'a, Node<T>, compt::dfs_order::PreOrder>,
}

impl<'a, T:HasAabbMut> VistrMut<'a, T> {
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    #[inline]
    pub fn create_wrap_mut(&mut self) -> VistrMut<T> {
        VistrMut {
            inner: self.inner.create_wrap_mut(),
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        //Safe since we know Vistr implements FixedDepthVisitor.
        self.inner.level_remaining_hint().0
    }



}


impl<'a, T:HasAabbMut> core::ops::Deref for VistrMut<'a, T> {
    type Target = Vistr<'a, T>;
    #[inline]
    fn deref(&self) -> &Vistr<'a, T> {
        unsafe { &*(self as *const VistrMut<T> as *const Vistr<T>) }
    }
}



unsafe impl<'a, T:HasAabbMut> compt::FixedDepthVisitor for VistrMut<'a, T> {}

impl<'a, T:HasAabbMut> Visitor for VistrMut<'a, T> {
    type Item = NodeRefMut<'a, T>;

    #[inline]
    fn next(self) -> (Self::Item, Option<[Self; 2]>) {
        let (nn, rest) = self.inner.next();

        let k = match rest {
            Some([left, right]) => Some([VistrMut { inner: left }, VistrMut { inner: right }]),
            None => None,
        };
        (nn.get_mut(), k)
    }
    #[inline]
    fn level_remaining_hint(&self) -> (usize, Option<usize>) {
        self.inner.level_remaining_hint()
    }


    #[inline]
    fn dfs_preorder(self,mut func:impl FnMut(Self::Item)){
        self.inner.dfs_preorder(|a|{
            func(a.get_mut())
        });
    }
}


///A node in a dinotree.
pub(crate) struct Node<T: HasAabb> {
    pub(crate) range: tools::Unique<ElemSlice<T>>,

    //range is empty iff cont is none.
    pub(crate) cont: axgeom::Range<T::Num>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    pub(crate) div: Option<T::Num>,
}






///Mutable reference to a node in the dinotree.
pub struct NodeRefMut<'a, T:HasAabbMut> {
    ///The bots that belong to this node.
    pub bots: ElemSliceMut<'a,T>,//&'a mut ElemSlice<T>,

    ///Is None iff bots is empty.
    pub cont: Option<&'a axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: Option<&'a T::Num>,
}


///Reference to a node in the dinotree.
pub struct NodeRef<'a, T:HasAabb> {
    ///The bots that belong to this node.
    pub bots: &'a ElemSlice<T>,

    ///Is None iff bots is empty.
    pub cont: Option<&'a axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: Option<&'a T::Num>,
}



impl<T: HasAabbMut> Node<T> {

    
    
    #[inline]
    pub fn get_mut(&mut self) -> NodeRefMut<T> {
        let bots = unsafe { &mut *self.range.as_ptr() };
        let cont = if bots.is_empty() {
            None
        } else {
            Some(&self.cont)
        };

        NodeRefMut {
            bots:ElemSliceMut::new(bots),
            cont,
            div: self.div.as_ref(),
        }
    }
    
    #[inline]
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


    pub(crate) fn create_tree_seq<A:AxisTrait,T:HasAabbMut,K:Splitter>(
            div_axis: A,
            rest: &mut [T],
            sorter: impl Sorter,
            splitter: &mut K,
            height: usize,
            binstrat: BinStrat,
            ) -> compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>{
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
            .fold(0, |acc, a| acc + a.get().bots.len());
        debug_assert_eq!(k, num_bots);

        tree
    }
    pub(crate) fn create_tree_par<A:AxisTrait,JJ:par::Joiner,T:HasAabbMut+Send+Sync,K:Splitter+Send+Sync>(
            div_axis: A,
            dlevel: JJ,
            rest: &mut [T],
            sorter: impl Sorter,
            splitter: &mut K,
            height: usize,
            binstrat: BinStrat,
            ) -> compt::dfs_order::CompleteTreeContainer<Node<T>, compt::dfs_order::PreOrder>{
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
            .fold(0, |acc, a| acc + a.get().bots.len());
        debug_assert_eq!(k, num_bots);

        tree
    }

    struct Recurser<'a, T: HasAabbMut, K: Splitter, S: Sorter> {
        height: usize,
        binstrat: BinStrat,
        sorter: S,
        _p :PhantomData<(K,&'a T)>
    }


    impl<'a, T: HasAabbMut, K: Splitter , S: Sorter> Recurser<'a, T, K, S> {

        fn create_leaf<A:AxisTrait>(&self,axis:A,rest:&'a mut [T]) -> Node<T>{
            let cont = if !rest.is_empty() {
                self.sorter.sort(axis.next(), rest);
                create_cont(axis, rest)
            } else {
                //We use unsafe here since we don't want to add more type contraints
                //on NumTrait. If we add Default, then it becomes hard to implement
                //some number types that count the number of comparisions made.

                //It is safe to do, since this cont will never be accessed.
                //Before we return a NodeRef, or NodeRefMut, we check if there are
                //any bots in the node, and only then return cont.
                unsafe{core::mem::MaybeUninit::zeroed().assume_init()}
            };

            Node {
                range: tools::Unique::new(ElemSlice::from_slice_mut(rest) as *mut _).unwrap(),
                cont,
                div: None,
            }
        }

        fn create_non_leaf<'b,A:AxisTrait>(&self,axis:A,rest:&'b mut [T]) -> (Node<T>,&'b mut [T],&'b mut [T]){
            match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
                ConstructResult::NonEmpty {
                    cont,
                    div,
                    mid,
                    left,
                    right,
                } => (
                    Node {
                        range: tools::Unique::new(ElemSlice::from_slice_mut(mid) as *mut _).unwrap(),
                        cont,
                        div: Some(div),
                    },
                    left,
                    right,
                ),
                ConstructResult::Empty(empty) => {

                    let (a,empty) = tools::duplicate_empty_slice(empty);
                    let (b,c) = tools::duplicate_empty_slice(empty);
                        
                    let node = Node {
                        range: tools::Unique::new(ElemSlice::from_slice_mut(a) as *mut _).unwrap(),
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
            nodes: &mut Vec<Node<T>>,
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
    impl<'a, T: HasAabbMut + Send + Sync, K: Splitter + Send+ Sync , S: Sorter> Recurser<'a, T, K, S> {



        fn recurse_preorder<A: AxisTrait, JJ: par::Joiner>(
            &self,
            axis: A,
            dlevel: JJ,
            rest: &'a mut [T],
            nodes: &mut Vec<Node<T>>,
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
                                let mut nodes2: Vec<Node<T>> =
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

fn create_cont<A: AxisTrait, T: HasAabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
    let (first, rest) = middle.split_first().unwrap();

    let mut min = first.get().rect.get_range(axis).left;
    let mut max = first.get().rect.get_range(axis).right;

    for a in rest.iter() {
        let left = &a.get().rect.get_range(axis).left;
        let right = &a.get().rect.get_range(axis).right;

        if *left < min {
            min = *left;
        }

        if *right > max {
            max = *right;
        }
    }

    axgeom::Range {
        left: min,
        right: max,
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
        cont: axgeom::Range<T::Num>,
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

        k.get().rect.get_range(div_axis).left
    };

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

    debug_assert!(!binned.middle.is_empty());

    sorter.sort(div_axis.next(), binned.middle);

    //We already know that the middile is non zero in length.
    let cont = create_cont(div_axis, binned.middle);

    ConstructResult::NonEmpty {
        mid: binned.middle,
        cont,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
