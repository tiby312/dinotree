use inner_prelude::*;

use tree::dinotree_simple;
use tree::dinotree_advanced;






enum VistrEnum<'a,N,T:HasAabb>{
    Simple(dinotree_simple::Vistr<'a,N,T>),
    Advanced(dinotree_advanced::Vistr<'a,N,T>)
}


enum VistrMutEnum<'a,N,T:HasAabb>{
    Simple(dinotree_simple::VistrMut<'a,N,T>),
    Advanced(dinotree_advanced::VistrMut<'a,N,T>)
}


pub struct Vistr<'a,N:'a,T:HasAabb+'a>{
	inner:VistrEnum<'a,N,T>
}

impl<'a,N:'a,T:HasAabb+'a> Vistr<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->Vistr<'b,N,T>{
        match &self.inner{
            VistrEnum::Simple(a)=>Vistr{inner:VistrEnum::Simple(a.create_wrap())},
            VistrEnum::Advanced(a)=>Vistr{inner:VistrEnum::Advanced(a.create_wrap())}
        }
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for Vistr<'a,N,T>{}

impl<'a,N:'a,T:HasAabb+'a> Visitor for Vistr<'a,N,T>{
    type Item=NodeRef<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        match self.inner{
            VistrEnum::Simple(a)=>{
                let (n,rest)=a.next();
                let k=match rest{
                    Some((f,left,right))=>{
                        let left=Vistr{inner:VistrEnum::Simple(left)};
                        let right=Vistr{inner:VistrEnum::Simple(right)};
                        Some((f,left,right))
                    },
                    None=>{
                        None
                    }
                };
                (n,k)
            },
            VistrEnum::Advanced(a)=>{
                let (n,rest)=a.next();
                let k=match rest{
                    Some((f,left,right))=>{
                        let left=Vistr{inner:VistrEnum::Advanced(left)};
                        let right=Vistr{inner:VistrEnum::Advanced(right)};
                        Some((f,left,right))
                    },
                    None=>{
                        None
                    }
                };
                (n,k)
            }
        }



    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        match &self.inner{
            VistrEnum::Simple(a)=>a.level_remaining_hint(),
            VistrEnum::Advanced(a)=>a.level_remaining_hint()
        }
    }
}





/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct VistrMut<'a,N:'a,T:HasAabb+'a>{
	inner:VistrMutEnum<'a,N,T>
}

impl<'a,N:'a,T:HasAabb+'a> VistrMut<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        match &mut self.inner{
            VistrMutEnum::Simple(a)=>VistrMut{inner:VistrMutEnum::Simple(a.create_wrap_mut())},
            VistrMutEnum::Advanced(a)=>VistrMut{inner:VistrMutEnum::Advanced(a.create_wrap_mut())}
        }
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for VistrMut<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Visitor for VistrMut<'a,N,T>{
    type Item=NodeRefMut<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        match self.inner{
            VistrMutEnum::Simple(a)=>{
                let (n,rest)=a.next();
                let k=match rest{
                    Some((f,left,right))=>{
                        let left=VistrMut{inner:VistrMutEnum::Simple(left)};
                        let right=VistrMut{inner:VistrMutEnum::Simple(right)};
                        Some((f,left,right))
                    },
                    None=>{
                        None
                    }
                };
                (n,k)
            },
            VistrMutEnum::Advanced(a)=>{
                let (n,rest)=a.next();
                let k=match rest{
                    Some((f,left,right))=>{
                        let left=VistrMut{inner:VistrMutEnum::Advanced(left)};
                        let right=VistrMut{inner:VistrMutEnum::Advanced(right)};
                        Some((f,left,right))
                    },
                    None=>{
                        None
                    }
                };
                (n,k)
            }
        }
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        match &self.inner{
            VistrMutEnum::Simple(a)=>a.level_remaining_hint(),
            VistrMutEnum::Advanced(a)=>a.level_remaining_hint()
        }
    }
}



pub enum DinoTreeType{
	Simple(),
	Advanced()
}


enum DinoTreeEnum<A:AxisTrait,N,T:HasAabb>{
	Simple(dinotree_simple::DinoTree<A,N,T>),
	Advanced(dinotree_advanced::DinoTree<A,N,T>)
}

impl<A:AxisTrait,N,T:HasAabb> DinoTreeEnum<A,N,T>{
     ///Create a mutable tree visitor.
    #[inline]
    pub fn vistr_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        match self{
            DinoTreeEnum::Simple(a)=>VistrMut{inner:VistrMutEnum::Simple(a.vistr_mut())},
            DinoTreeEnum::Advanced(a)=>VistrMut{inner:VistrMutEnum::Advanced(a.vistr_mut())}
        }
    }
}
/*


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





fn create_tree_iter_mut<'a,N,T:HasAabb>(vistrmut:VistrMut<'a,N,T>,num_bots:usize)->iter_mut::TreeIterMut<'a,N,T>{
    let it=vistrmut.dfs_inorder_iter().flat_map(iter_mut::convert as iter_mut::FF<N,T>);
    iter_mut::TreeIterMut{it:unsafe{CustomLength::new(it,num_bots)}}
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
*/


pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
	inner:DinoTreeEnum<A,N,T>,
    mover:Vec<u32>
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{

	pub(crate) fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
	    rebal_type:RebalStrat,axis:A,n:N,bots:&[T],aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter,dinotree:Option<DinoTreeType>)->DinoTree<A,N,BBox<Num,T>>
	{   
        
        let dinotree=match dinotree{
            Some(x)=>x,
            None=>DinoTreeType::Simple()
        };


        let (inner,mover)=match dinotree{
            DinoTreeType::Simple()=>{
                let (t,mover)=dinotree_simple::DinoTree::new(rebal_type,axis,n,bots,aabb_create,ka,height,par,sorter);

                (DinoTreeEnum::Simple(t),mover)
            },
            DinoTreeType::Advanced()=>{
                let (t,mover)=dinotree_advanced::DinoTree::new(rebal_type,axis,n,bots,aabb_create,ka,height,par,sorter);
                (DinoTreeEnum::Advanced(t),mover)
            }   
        };

        DinoTree{inner,mover}
	}

    
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

        Self::new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,dlevel,DefaultSorter,None)
    }

    #[inline]
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;
        Self::new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,par::Sequential,DefaultSorter,None)
    }
    
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

        let treev=self.inner.vistr_mut().dfs_inorder_iter().flat_map(|(a,_)|a.range.iter_mut());
    
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
    pub fn iter_mut<'a>(&'a mut self)->impl Iterator<Item=&mut T>{
        self.vistr_mut().dfs_inorder_iter().flat_map(|(a,_)|a.range.iter_mut())
    }

    ///See iter_mut
    #[inline]
    pub fn iter<'a>(&'a self)->impl Iterator<Item=&T>{
        self.vistr().dfs_inorder_iter().flat_map(|(a,_)|a.range.iter())
    }
    
    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    #[inline]
    pub fn axis(&self)->A{
        match &self.inner{
            DinoTreeEnum::Simple(a)=>a.axis(),
            DinoTreeEnum::Advanced(a)=>a.axis()
        }
    }

    ///Get the height of the tree.
    #[inline]
    pub fn height(&self)->usize{
        match &self.inner{
            DinoTreeEnum::Simple(a)=>a.height(),
            DinoTreeEnum::Advanced(a)=>a.height()
        }
    }

    ///Create a mutable tree visitor.
    #[inline]
    pub fn vistr_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        self.inner.vistr_mut()
    }

    ///Create an immutable tree visitor.
    #[inline]
    pub fn vistr<'b>(&'b self)->Vistr<'b,N,T>{
        match &self.inner{
            DinoTreeEnum::Simple(a)=>Vistr{inner:VistrEnum::Simple(a.vistr())},
            DinoTreeEnum::Advanced(a)=>Vistr{inner:VistrEnum::Advanced(a.vistr())}
        }
    }

    ///Returns the number of bots that are in the tree.
    #[inline]
    pub fn num_bots(&self)->usize{
        match &self.inner{
            DinoTreeEnum::Simple(a)=>a.num_bots(),
            DinoTreeEnum::Advanced(a)=>a.num_bots()
        }

    }

    ///Returns the number of nodes in the tree.
    #[inline]
    pub fn num_nodes(&self)->usize{
        match &self.inner{
            DinoTreeEnum::Simple(a)=>a.num_nodes(),
            DinoTreeEnum::Advanced(a)=>a.num_nodes()
        }
    }
}

