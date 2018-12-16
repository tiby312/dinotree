

pub mod dinotree;
pub mod dinotree_no_copy;
pub mod notsorted;

use crate::inner_prelude::*;




///Mutable referance to a dinotree container.
pub struct DinoTreeRefMut<'a,A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:&'a mut [T],
    tree:&'a mut compt::dfs_order::CompleteTree<Node3<N,T>,compt::dfs_order::InOrder>
}

impl<'a,A:AxisTrait,N,T:HasAabb> DinoTreeRefMut<'a,A,N,T>{

    pub fn as_ref_mut(&mut self)->DinoTreeRefMut<A,N,T>{
        DinoTreeRefMut{axis:self.axis,bots:self.bots,tree:self.tree}
    }

    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        VistrMut{inner:self.tree.vistr_mut()}
    }

    pub fn into_vistr_mut(self)->VistrMut<'a,N,T>{
        VistrMut{inner:self.tree.vistr_mut()}
    }

    ///Iterate over al the bots in the tree. The order in which they are iterated is dfs in order.
    #[inline]
    pub fn iter_mut(&mut self)->std::slice::IterMut<T>{
        self.bots.iter_mut()
    }

    #[inline]
    pub fn into_iter_mut(self)->std::slice::IterMut<'a,T>{
        self.bots.iter_mut()
    }
}

impl<'a,A:AxisTrait,N,T:HasAabb> std::ops::Deref for DinoTreeRefMut<'a,A,N,T>{
    type Target=DinoTreeRef<'a,A,N,T>;
    fn deref(&self)->&DinoTreeRef<'a,A,N,T>{
        unsafe{&*(self as *const tree::DinoTreeRefMut<'a, A, N, T> as *const tree::DinoTreeRef<'a, A, N, T>)}
        //unsafe{std::mem::transmute(self)}
    }
}


///Referance to a dinotree container.
pub struct DinoTreeRef<'a,A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:&'a [T],
    tree:&'a compt::dfs_order::CompleteTree<Node3<N,T>,compt::dfs_order::InOrder>
}

impl<'a,A:AxisTrait,N,T:HasAabb> DinoTreeRef<'a,A,N,T>{
    pub fn as_ref(&self)->DinoTreeRef<A,N,T>{
        DinoTreeRef{axis:self.axis,bots:self.bots,tree:self.tree}
    }

    pub fn axis(&self)->A{
        self.axis
    }

    pub fn into_vistr(self)->Vistr<'a,N,T>{
        Vistr{inner:self.tree.vistr()}
    }

    pub fn vistr(&self)->Vistr<N,T>{
        Vistr{inner:self.tree.vistr()}
    }

    ///See iter_mut
    #[inline]
    pub fn iter(&self)->std::slice::Iter<T>{
        self.bots.iter()
    }

    #[inline]
    pub fn into_iter(self)->std::slice::Iter<'a,T>{
        self.bots.iter()
    }


    #[inline]
    pub fn height(&self)->usize{
        self.tree.get_height()
    }
    #[inline]
    pub fn num_nodes(&self)->usize{
        self.tree.get_nodes().len()
    }

    #[inline]
    pub fn num_bots(&self)->usize{
        self.bots.len()
    }
}



///Outputs the height given an desirned number of bots per node.
#[inline]
pub fn compute_tree_height_heuristic_debug(num_bots: usize,num_per_node:usize) -> usize {
    
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        (num_bots as f32 / num_per_node as f32).log2().ceil() as usize
    }
}


///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn default_level_switch_sequential()->usize{
    const DEPTH_SEQ:usize=4;
    DEPTH_SEQ
}

///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn compute_default_level_switch_sequential(depth:usize,height:usize)->par::Parallel{
    const DEPTH_SEQ:usize=4;
    let dd=depth;
    
    let gg=if height<=dd{
        0
    }else{
        height-dd
    };
    
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
    //const NUM_PER_NODE: usize = 32;  
    const NUM_PER_NODE: usize = 20;  


    if num_bots <= NUM_PER_NODE {
        1
    } else {
        (num_bots as f32 / NUM_PER_NODE as f32).log2().ceil() as usize
    }
}





/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct Vistr<'a,N:'a,T:HasAabb+'a>{
    inner:compt::dfs_order::Vistr<'a,Node3<N,T>,compt::dfs_order::InOrder>
}

impl<'a,N:'a,T:HasAabb+'a> Vistr<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap(&self)->Vistr<N,T>{
        Vistr{inner:self.inner.create_wrap()}
    }

    pub fn height(&self)->usize{
        //Safe since we know Vistr implements FixedDepthVisitor.
        self.inner.level_remaining_hint().0
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for Vistr<'a,N,T>{}

impl<'a,N:'a,T:HasAabb+'a> Visitor for Vistr<'a,N,T>{
    type Item=NodeRef<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        let (nn,rest)=self.inner.next();
        
        let k=match rest{
            Some(((),left,right))=>{
                let f=match &nn.fullcomp{
                    FullCompOrEmpty::NonEmpty(f)=>{
                        Some(f)
                    },
                    FullCompOrEmpty::Empty()=>{
                        None
                    }
                };
                Some((f,Vistr{inner:left},Vistr{inner:right}))
            },
            None=>{
                None
            }
        };

        let j=NodeRef{misc:&nn.n,range:unsafe{nn.mid.as_ref()}};
        (j,k)


    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}





/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct VistrMut<'a,N:'a,T:HasAabb+'a>{
    inner:compt::dfs_order::VistrMut<'a,Node3<N,T>,compt::dfs_order::InOrder>
}

impl<'a,N:'a,T:HasAabb+'a> VistrMut<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut(&mut self)->VistrMut<N,T>{
        VistrMut{inner:self.inner.create_wrap_mut()}
    }


    pub fn height(&self)->usize{
        //Safe since we know Vistr implements FixedDepthVisitor.
        self.inner.level_remaining_hint().0
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for VistrMut<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Visitor for VistrMut<'a,N,T>{
    type Item=NodeRefMut<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        let (nn,rest)=self.inner.next();
        
        let k=match rest{
            Some(((),left,right))=>{
                let f=match &nn.fullcomp{
                    FullCompOrEmpty::NonEmpty(f)=>{
                        Some(f)
                    },
                    FullCompOrEmpty::Empty()=>{
                        None
                    }
                };
                Some((f,VistrMut{inner:left},VistrMut{inner:right}))
            },
            None=>{
                None
            }
        };

        let j=NodeRefMut{misc:&mut nn.n,range:unsafe{nn.mid.as_mut()}};
        (j,k)


    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}
pub struct Node3<N,T:HasAabb>{ 
    pub n:N,
    pub fullcomp:FullCompOrEmpty<T::Num>,
    pub mid:std::ptr::Unique<[T]>
}



/// Reference to a node returned by the Vistr tree iterator.
pub struct NodeRef<'a,N,T>{
    pub misc:&'a N,
    pub range:&'a [T]
}

/// Reference to a node returned by the VistrMut tree iterator.
pub struct NodeRefMut<'a,N,T>{
    pub misc:&'a mut N,
    pub range:&'a mut [T]
}



pub trait Sorter:Copy+Clone+Send+Sync{
    fn sort(&self,axis:impl AxisTrait,bots:&mut [impl HasAabb]);
}

#[derive(Copy,Clone)]
pub struct DefaultSorter;

impl Sorter for DefaultSorter{
    fn sort(&self,axis:impl AxisTrait,bots:&mut [impl HasAabb]){
        oned::sweeper_update(axis,bots);
    }
}

#[derive(Copy,Clone)]
pub struct NoSorter;

impl Sorter for NoSorter{
    fn sort(&self,_axis:impl AxisTrait,_bots:&mut [impl HasAabb]){}
}


///A struct that contains data that only non leaf nodes contain.
#[derive(Copy,Clone)]
pub struct FullComp<N:NumTrait>{
    ///The position of the splitting line for this node.
    pub div:N,
    ///The 1d bounding box for this node. All bots that intersect the splitting line are 
    ///within this bounding box.
    pub cont:axgeom::Range<N> ,

}

#[derive(Copy,Clone)]
pub enum FullCompOrEmpty<N:NumTrait>{
    NonEmpty(FullComp<N>),
    Empty()
}




fn nodes_left(depth:usize,height:usize)->usize{
    let levels=height-depth;
    2usize.rotate_left(levels as u32)-1
}


mod cont_tree{

    use super::*;


    pub struct Node2<'a,Num:NumTrait+'a>{ 

        //If this is a non leaf node, then,
        //  div is None iff this node and children nodes do not have any bots in them.
        // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
        // This is because we specifically pick the median.
        // If it is a leaf node, then div being none still means it could have bots in it.
        pub fullcomp:FullCompOrEmpty<Num>,
        pub mid:&'a mut [Cont2<Num>]
    }




    #[derive(Copy,Clone)]
    pub struct Cont2<N:NumTrait>{
        pub rect:Rect<N>,
        pub index:u32
    }
    unsafe impl<N:NumTrait> HasAabb for Cont2<N>{
        type Num=N;
        fn get(&self)->&Rect<N>{
            &self.rect
        }
    }   





    pub struct ContTree<'a,Num:NumTrait>{
        tree:compt::dfs_order::CompleteTreeContainer<Node2<'a,Num>,compt::dfs_order::InOrder>,
        conts:&'a mut [Cont2<Num>]
    }

    impl<'a,Num:NumTrait> ContTree<'a,Num>{
        pub fn get_tree_mut(&mut self)->&mut compt::dfs_order::CompleteTree<Node2<'a,Num>,compt::dfs_order::InOrder>{
            &mut self.tree
        }
        pub fn get_tree(&self)->&compt::dfs_order::CompleteTree<Node2<'a,Num>,compt::dfs_order::InOrder>{
            &self.tree
        }
        pub fn get_conts_mut(&mut self)->&mut [Cont2<Num>]{
            self.conts
        }
        pub fn get_conts(&self)->&[Cont2<Num>]{
            self.conts
        }

        pub fn new<A:AxisTrait,JJ:par::Joiner,K:Splitter+Send>(div_axis:A,dlevel:JJ,rest:&'a mut [Cont2<Num>],sorter:impl Sorter,splitter:&mut K,height:usize,binstrat:BinStrat)->ContTree<'a,Num>{
            let rest2=unsafe{&mut *(rest as *mut [_])};
            let mut nodes=Vec::with_capacity(tree::nodes_left(0,height));
                
            let r=Recurser{height,binstrat,sorter,_p:PhantomData};
            r.recurse(div_axis,dlevel,rest,&mut nodes,splitter,0);
            //recurse_rebal(div_axis,dlevel,rest,&mut nodes,sorter,splitter,0,height,binstrat);

            let tree=compt::dfs_order::CompleteTreeContainer::from_vec(nodes).unwrap();
            ContTree{tree,conts:rest2}
        }
    }



    struct Recurser<'a,Num:NumTrait,K:Splitter+Send,S:Sorter>{
        height:usize,
        binstrat:BinStrat,
        sorter:S,
        _p:PhantomData<(std::sync::Mutex<K>,&'a (Num))>
    }


    impl<'a,Num:NumTrait,K:Splitter+Send,S:Sorter> Recurser<'a,Num,K,S>{

        fn recurse<A:AxisTrait,JJ:par::Joiner>(
            &self,
            axis:A,
            dlevel:JJ,
            rest:&'a mut [Cont2<Num>],
            nodes:&mut Vec<Node2<'a,Num>>,
            splitter:&mut K,
            depth:usize
            )
        {
            splitter.node_start();

            if depth<self.height-1{
                

                let mut splitter2=splitter.div();


                let (node,left,right)=match construct_non_leaf(self.binstrat,self.sorter,axis,rest){
                    Some((fullcomp,left,mid,right))=>{
                        
                        (Node2{fullcomp:FullCompOrEmpty::NonEmpty(fullcomp),mid},left,right)
                    },
                    None=>{
                        //We don't want to return here since we still want to populate the whole tree!
                        (Node2{fullcomp:FullCompOrEmpty::Empty(),mid:&mut []},&mut [] as &mut [_],&mut [] as &mut [_])
                    }
                };
                
                let splitter=if !dlevel.should_switch_to_sequential(Depth(depth)){
                    let splitter2=&mut splitter2;

                    let af= move || {
                        self.recurse(axis.next(),dlevel,left,nodes,splitter,depth+1);
                        (splitter,nodes)
                    };

                    let bf= move || {

                        let mut nodes2:Vec<Node2<'a,Num>>=Vec::with_capacity(nodes_left(depth,self.height));
                        nodes2.push(node);                
                        self.recurse(axis.next(),dlevel,right,&mut nodes2,splitter2,depth+1);
                        nodes2
                    };
                    let ((splitter,nodes),mut nodes2)=rayon::join(af,bf);
                    nodes.append(&mut nodes2);
                    splitter
                }else{
                    self.recurse(axis.next(),dlevel.into_seq(),left,nodes,splitter,depth+1);
                    nodes.push(node);
                    self.recurse(axis.next(),dlevel.into_seq(),right,nodes,&mut splitter2,depth+1);
                    splitter
                };
                
                splitter.add(splitter2);
            }else{
                let mut node=Node2{fullcomp:FullCompOrEmpty::Empty(),mid:rest};
                construct_leaf(self.sorter,axis,&mut node.mid);
                nodes.push(node);
                splitter.node_end();
            }            

        }
    }

    
}






pub fn create_cont<A:AxisTrait,T:HasAabb>(axis:A,middile:&[T])->Option<axgeom::Range<T::Num>>{
  

    let (first,rest)=middile.split_first()?;
    let mut min=first.get().get_range(axis).left;
    let mut max=first.get().get_range(axis).right;

    for a in rest.iter(){
        let left=a.get().get_range(axis).left;
        let right=a.get().get_range(axis).right;

        if left<min{
            min=left;
        }

        if right>max{
            max=right;
        }
    }

    Some(axgeom::Range{left:min,right:max})

}



pub fn construct_leaf<T:HasAabb>(sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T]){ 
    sorter.sort(div_axis.next(),bots);
}


#[allow(dead_code)]
#[derive(Copy,Clone,Debug)]
pub enum BinStrat{
    LeftMidRight,
    LeftMidRightUnchecked,
    MidLeftRight,
    LeftRightMid
}

pub fn construct_non_leaf<T:HasAabb>(bin_strat:BinStrat,sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T])->Option<(FullComp<T::Num>,&mut [T],&mut [T],&mut [T])>{
    let med=if bots.is_empty(){
        return None;
    }
    else
    {
        let closure = |a: &T, b: &T| -> std::cmp::Ordering {
            oned::compare_bots(div_axis,a,b)
        };

        let k={
            let mm=bots.len()/2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).left
    };

    
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned=match bin_strat{
        BinStrat::LeftMidRight=>{
            oned::bin_left_middle_right(div_axis,&med,bots)
        },
        BinStrat::LeftMidRightUnchecked=>{
            unsafe{oned::bin_left_middle_right_unchecked(div_axis,&med,bots)}
        },
        BinStrat::MidLeftRight=>{
            oned::bin_middle_left_right(div_axis,&med,bots)
        },
        BinStrat::LeftRightMid=>{
            oned::bin_left_right_middle(div_axis,&med,bots)
        }
    };

    debug_assert!(!binned.middle.is_empty());
    
    
    sorter.sort(div_axis.next(),binned.middle);
    
    //We already know that the middile is non zero in length.
    let container_box=create_cont(div_axis,binned.middle).unwrap();
    
    let full=FullComp{div:med,cont:container_box};
    Some((full,binned.left,binned.middle,binned.right))
}



///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user 
///cannot modify the bounding box since it is hidden, with only read access.
#[derive(Copy,Clone)]
pub struct BBox<N:NumTrait,T>{
    rect:Rect<N>,
    pub inner:T
}

use std::fmt::Formatter;
use std::fmt::Debug;

impl<N:NumTrait+Debug,T:Debug> Debug for BBox<N,T>{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result{
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

impl<N:NumTrait,T> BBox<N,T>{
    ///Unsafe since the user create to boxes whose rectangles do not intersect,
    ///but whose contents point to a shared resource thus violating the contract of HasAabb.
    #[inline]
    pub unsafe fn new(rect:Rect<N>,inner:T)->BBox<N,T>{
        BBox{rect,inner}
    }

}

unsafe impl<N:NumTrait,T> HasAabb for BBox<N,T>{
    type Num=N;
    #[inline]
    fn get(&self)->&Rect<Self::Num>{
        &self.rect
    }
}


///Some alternate strategies for rebalancing.
///Used to empirically show that the default (the first one) is a good default.
#[derive(Debug)]
pub enum RebalStrat{
    First,
    Second,
    Third
}




