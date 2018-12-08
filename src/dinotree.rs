use inner_prelude::*;


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





#[derive(Copy,Clone)]
pub(crate)struct Cont2<N:NumTrait>{
    rect:Rect<N>,
    pub index:u32
}
unsafe impl<N:NumTrait> HasAabb for Cont2<N>{
    type Num=N;
    fn get(&self)->&Rect<N>{
        &self.rect
    }
}   






mod dinotree2{
    pub use super::*;

    pub struct Vistr2<'a,N:'a,T:HasAabb+'a>{
        inner:compt::dfs_order::Vistr<'a,Node3<N,T>>
    }

    impl<'a,N:'a,T:HasAabb+'a> Vistr2<'a,N,T>{
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        pub fn create_wrap<'b>(&'b mut self)->Vistr2<'b,N,T>{
            Vistr2{inner:self.inner.create_wrap()}
        }
    }

    unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for Vistr2<'a,N,T>{}
    impl<'a,N:'a,T:HasAabb+'a> Visitor for Vistr2<'a,N,T>{
        type Item=(&'a N,&'a [T]);
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
                    Some((f,Vistr2{inner:left},Vistr2{inner:right}))
                },
                None=>{
                    None
                }
            };

            ((&nn.n,unsafe{nn.mid.as_ref()}),k)


        }
        fn level_remaining_hint(&self)->(usize,Option<usize>){
            self.inner.level_remaining_hint()
        }
    }


    /// Tree Iterator that returns a reference to each node.
    /// It also returns the non-leaf specific data when it applies.
    pub struct VistrMut2<'a,N:'a,T:HasAabb+'a>{
        inner:compt::dfs_order::VistrMut<'a,Node3<N,T>>
    }

    impl<'a,N:'a,T:HasAabb+'a> VistrMut2<'a,N,T>{
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut2<'b,N,T>{
            VistrMut2{inner:self.inner.create_wrap_mut()}
        }
    }

    unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for VistrMut2<'a,N,T>{}
    impl<'a,N:'a,T:HasAabb+'a> Visitor for VistrMut2<'a,N,T>{
        type Item=(&'a mut N,&'a mut [T]);
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
                    Some((f,VistrMut2{inner:left},VistrMut2{inner:right}))
                },
                None=>{
                    None
                }
            };

            ((&mut nn.n,unsafe{nn.mid.as_mut()}),k)


        }
        fn level_remaining_hint(&self)->(usize,Option<usize>){
            self.inner.level_remaining_hint()
        }
    }


    pub struct Node3<N,T:HasAabb>{ 
        pub n:N,
        //If this is a non leaf node, then,
        //  div is None iff this node and children nodes do not have any bots in them.
        // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
        // This is because we specifically pick the median.
        // If it is a leaf node, then div being none still means it could have bots in it.
        pub fullcomp:FullCompOrEmpty<T::Num>,
        pub mid:std::ptr::Unique<[T]>
    }

    pub struct DinoTree2<A,N,T:HasAabb>{
        axis:A,
        bots:Vec<T>,
        nodes:compt::dfs_order::CompleteTree<Node3<N,T>>,
        mover:Vec<u32>,
    }

    pub fn new_inner2<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(
        rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree2<A,N,BBox<Num,T>>
    {   
        let num_bots=bots.len();
        let max=std::u32::MAX;
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let conts=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
                    });



        let mut conts:Vec<_>=conts.collect();
        
        let mut nodes=Vec::new();
        dinotree_inner::recurse_rebal1(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);


        let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();

        let mut new_bots:Vec<BBox<Num,T>>=Vec::with_capacity(num_bots);
        for node in tree.dfs_inorder_iter(){
            for a in node.mid.iter(){
                new_bots.push(BBox{rect:a.rect,inner:bots[a.index as usize]});
            }
        }



        let new_nodes={
            let mut rest:Option<&mut [BBox<Num,T>]>=Some(&mut new_bots);
            let mut new_nodes=Vec::new();
            for node in tree.dfs_inorder_iter(){
                let (b,rest2)=rest.take().unwrap().split_at_mut(node.mid.len());
                rest=Some(rest2);
                
                let b=unsafe{std::ptr::Unique::new_unchecked(b as *mut [_])};
                new_nodes.push(Node3{n,fullcomp:node.fullcomp,mid:b});
            }
            new_nodes
        };

        let tree2=compt::dfs_order::CompleteTree::from_vec(new_nodes,height).unwrap();


        let mut nodes=tree.into_nodes();

        let mover={
            let mut mover=Vec::with_capacity(num_bots);
            for node in nodes.drain(..){
                mover.extend(node.mid.iter().map(|a|a.index));
            }
            mover
        };
        DinoTree2{axis,bots:new_bots,nodes:tree2,mover}    
    }
}


pub fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(
    rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree<A,N,BBox<Num,T>>
{   
     
    let num_bots=bots.len();
    let max=std::u32::MAX;
    assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


    let conts=bots.iter().enumerate().map(|(index,k)|{
        Cont2{rect:aabb_create(k),index:index as u32}
                });



    let mut conts:Vec<_>=conts.collect();
    
    let nodes=match rebal_type{
        RebalStrat::First=>{
            let mut nodes=Vec::new();
            dinotree_inner::recurse_rebal1(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
            nodes
        },
        RebalStrat::Second=>{
            let mut nodes=SmallVec::new();
            dinotree_inner::recurse_rebal2(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
            nodes.into_vec()
        },
        RebalStrat::Third=>{
            let mut nodes=Vec::new();
            dinotree_inner::recurse_rebal3(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
            nodes
        }
    };
    let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();
    let alloc=alloc::TreeInner::from_dfs_in_order1(axis,height,num_bots,tree.vistr().map(|item,nonleaf|{
        let a=item.mid.iter().map(|a|BBox{rect:a.rect,inner:bots[a.index as usize]});
        
        let b=match nonleaf{
            Some(())=>{
                Some(item.fullcomp)
            },
            None=>{
                None
            }
        };
        (a,b)
    }),n);

    let mut nodes= tree.into_nodes();

    let mover={
        let mut mover=Vec::with_capacity(num_bots);
        for node in nodes.drain(..){
            mover.extend(node.mid.iter().map(|a|a.index));
        }
        mover
    };

    
    let tree=DinoTree{mover,alloc};

    tree
    
}


/// The tree this crate revoles around.
///
/// The user supplies a list of objects to insert along with a way to create a bounding box for each object.
/// Then the tree is constructed. The user does not have to supply a list of objects that implement HasAabb.
/// This was done deliberately to allow for designs where the bounding box is only created for each bot
/// at the time the tree is constructed. This way the aabb is not taking up space if the list of bots inbetween
/// tree constructions. This would improve locality with algorithms that dont care about the object's aabbs.
///
/// In order to avoid a level of indirection, the bots are copied into a tree, and then copied back out. The algorithm ensures
/// That even though the ordering is different, this is a bijection between the two sets.
/// So we can safely hide this unsafety from the user.
///
/// Unsafety is used to construct the special variable node size tree structure that is populated with dsts.
///
/// The mutable reference to each element in the callback functions do not point to elements
/// in the user supplied slice of elements. The elements are internally copied directly into a tree structure
/// and then copied back into the slice at the end. So do not try storing the mutable references as pointers
/// in the callback functions since they would point to unallocated memory once the tree is destroyed. If you wish to
/// store some kind of reference to each of the bots, pass the tree objects that contain inside them an index representing
/// their position in the list and store those as pairs.
///
/// The type parameter N is a user defined struct that every element of the tree will have purely for use
/// in user defined algorithms. This is useful for algorithms that might need to store data on a node by node basis.
/// Having the data be directly in the tree instead of a seperate data structure improvies memory locality for the algorithm.
///
pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
    mover:Vec<u32>, //Used to return the aabb objects back to their original position
    alloc:TreeInner<A,T,N>,
}


impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{


    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    #[inline]
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{  
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;


        //TODO simplify this code!!!
        //See the data project for reasoning behind this value.
        const DEPTH_SEQ:usize=2;

        let gg=if height<=DEPTH_SEQ{
            0
        }else{
            height-DEPTH_SEQ
        };
        
        let dlevel=par::Parallel::new(Depth(gg));

        new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,dlevel,DefaultSorter)
    }

    #[inline]
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;
        new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,par::Sequential,DefaultSorter)
    }

}


impl<A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTree<A,N,T>{
    
    ///Transform the current tree to have a different extra component to each node.
    pub fn with_extra<N2:Copy>(self,n2:N2)->DinoTree<A,N2,T>{
        
        let axis=self.axis();
        let height=self.height();
        let num_bots=self.num_bots();
        let mover=self.mover.clone();
        
        let alloc=alloc::TreeInner::from_dfs_in_order1(axis,height,num_bots,self.alloc.vistr().map(|item,nonleaf|{
            let a=item.range.iter().map(|a|*a);
            
            let b=match nonleaf{
                Some(fullcomp)=>{
                    match fullcomp{
                        Some(fullcomp)=>{
                            Some(FullCompOrEmpty::NonEmpty(*fullcomp))
                        },
                        None=>{
                            Some(FullCompOrEmpty::Empty())
                        }
                    }
                },
                None=>{
                    None
                }
            };

            (a,b)
        }),n2);


        DinoTree{alloc,mover}
    }
}





use self::customlength::CustomLength;
mod customlength{
    pub struct CustomLength<I:Iterator>{
        length:usize,
        num:usize,
        it:I
    }
    impl<I:Iterator> CustomLength<I>{
        pub unsafe fn new(it:I,length:usize)->CustomLength<I>{
            CustomLength{length,num:0,it}
        }
    }
    impl<I:Iterator> Iterator for CustomLength<I>{
        type Item=I::Item;
        fn next(&mut self)->Option<Self::Item>{
            self.num+=1;
            self.it.next()
        }
        fn size_hint(&self)->(usize,Option<usize>){
            (self.length-self.num,Some(self.length-self.num))
        }
    }
    impl<I:std::iter::FusedIterator> std::iter::FusedIterator for CustomLength<I>{}
    impl<I:Iterator> std::iter::ExactSizeIterator for CustomLength<I>{}
    unsafe impl<I:Iterator> std::iter::TrustedLen for CustomLength<I>{}
}


pub(crate) mod iter_mut{
    use super::*;

    pub fn convert<'a,N:'a,T:HasAabb+'a>(a:(&'a mut NodeDyn<N, T>,Option<Option<&'a FullComp<T::Num>>>))->std::slice::IterMut<'a,T>{
        a.0.range.iter_mut()
    }
    
    pub type FF<'a,N,T>=fn(  (&'a mut NodeDyn<N, T>,Option<Option<&'a FullComp<<T as HasAabb>::Num>>>) ) -> std::slice::IterMut<'a,T>;
    
    ///Iterator over all the elements in the tree in dfs in order- not the original order.
    pub struct TreeIterMut<'a,N:'a,T:HasAabb+'a>{
        pub(crate) it:CustomLength<std::iter::FlatMap<
            compt::DfsInOrderIter<VistrMut<'a,N,T>>,
            std::slice::IterMut<'a,T>,
            FF<'a,N,T>>>
    }
    impl<'a,N,T:HasAabb> Iterator for TreeIterMut<'a,N,T>{
        type Item=&'a mut T;
        fn next(&mut self)->Option<Self::Item>{
            self.it.next()
        }
        fn size_hint(&self)->(usize,Option<usize>){
            self.it.size_hint()
        }
    }

    impl<'a,N,T:HasAabb> std::iter::FusedIterator for TreeIterMut<'a,N,T>{}
    impl<'a,N,T:HasAabb> std::iter::ExactSizeIterator for TreeIterMut<'a,N,T>{}
    unsafe impl<'a,N,T:HasAabb> std::iter::TrustedLen for TreeIterMut<'a,N,T>{}
}



pub(crate) mod iter_const{
    use super::*;

    pub fn convert<'a,N:'a,T:HasAabb+'a>(a:(&'a NodeDyn<N, T>,Option<Option<&'a FullComp<T::Num>>>))->std::slice::Iter<'a,T>{
        a.0.range.iter()
    }
    
    pub type FF<'a,N,T>=fn(  (&'a NodeDyn<N, T>,Option<Option<&'a FullComp<<T as HasAabb>::Num>>>) ) -> std::slice::Iter<'a,T>;
    
    ///Iterator over all the elements in the tree in dfs in order- not the original order.
    pub struct TreeIter<'a,N:'a,T:HasAabb+'a>{
        pub(crate) it:CustomLength<std::iter::FlatMap<
            compt::DfsInOrderIter<Vistr<'a,N,T>>,
            std::slice::Iter<'a,T>,
            FF<'a,N,T>>>
    }
    impl<'a,N,T:HasAabb> Iterator for TreeIter<'a,N,T>{
        type Item=&'a T;
        fn next(&mut self)->Option<Self::Item>{
            self.it.next()
        }
        fn size_hint(&self)->(usize,Option<usize>){
            self.it.size_hint()
        }
    }

    impl<'a,N,T:HasAabb> std::iter::FusedIterator for TreeIter<'a,N,T>{}
    impl<'a,N,T:HasAabb> std::iter::ExactSizeIterator for TreeIter<'a,N,T>{}
    unsafe impl<'a,N,T:HasAabb> std::iter::TrustedLen for TreeIter<'a,N,T>{}
}



fn create_tree_iter_mut<'a,N,T:HasAabb>(vistrmut:VistrMut<'a,N,T>,num_bots:usize)->iter_mut::TreeIterMut<'a,N,T>{
    let it=vistrmut.dfs_inorder_iter().flat_map(iter_mut::convert as iter_mut::FF<N,T>);
    iter_mut::TreeIterMut{it:unsafe{CustomLength::new(it,num_bots)}}
}

impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{

    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    #[inline]
    pub fn apply<X>(&self,bots:&mut [X],conv:impl Fn(&T,&mut X)){
        assert_eq!(bots.len(),self.num_bots());
        for (bot,mov) in self.iter().zip_eq(self.mover.iter()){
            let target=&mut bots[*mov as usize];
            conv(bot,target);
        }
    }

    #[inline]
    pub fn apply_into<X>(&mut self,bots:&[X],conv:impl Fn(&X,&mut T)){
        assert_eq!(bots.len(),self.num_bots());

        let treev=create_tree_iter_mut(self.alloc.vistr_mut(),bots.len());
        for (bot,mov) in treev.zip_eq(self.mover.iter()){
            let source=&bots[*mov as usize];
            conv(source,bot)
        }
    }

    ///Iterate over al the bots in the tree. The order in which they are iterated is dfs in order.
    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    ///But this is useful if you need to iterate over all the bots aabbs.
    #[inline]
    pub fn iter_mut<'a>(&'a mut self)->iter_mut::TreeIterMut<'a,N,T>{
        let length=self.num_bots();
        create_tree_iter_mut(self.vistr_mut(),length)
    }

    ///See iter_mut
    #[inline]
    pub fn iter<'a>(&'a self)->iter_const::TreeIter<'a,N,T>{
        let length=self.num_bots();
        let it=self.vistr().dfs_inorder_iter().flat_map(iter_const::convert as iter_const::FF<N,T>);
        iter_const::TreeIter{it:unsafe{CustomLength::new(it,length)}}

    }
    
    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    #[inline]
    pub fn axis(&self)->A{
        self.alloc.axis()
    }

    ///Get the height of the tree.
    #[inline]
    pub fn height(&self)->usize{
        self.alloc.height()
    }

    ///Create a mutable tree visitor.
    #[inline]
    pub fn vistr_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        self.alloc.vistr_mut()
    }

    ///Create an immutable tree visitor.
    #[inline]
    pub fn vistr<'b>(&'b self)->Vistr<'b,N,T>{
        self.alloc.vistr()
    }

    ///Returns the number of bots that are in the tree.
    #[inline]
    pub fn num_bots(&self)->usize{
        self.alloc.num_bots()
    }

    ///Returns the number of nodes in the tree.
    #[inline]
    pub fn num_nodes(&self)->usize{
        self.alloc.num_nodes()
    }
}
