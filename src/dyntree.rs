use inner_prelude::*;
use base_kdtree::KdTree;
use HasAabb;
use tree_alloc::VistrMut;
use tree_alloc::Vistr;
use compt::Visitor;
use axgeom::*;
use advanced::Splitter;

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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result{
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

impl<N:NumTrait,T> BBox<N,T>{
    pub unsafe fn new(rect:Rect<N>,inner:T)->BBox<N,T>{
        BBox{rect,inner}
    }

}

unsafe impl<N:NumTrait,T> HasAabb for BBox<N,T>{
    type Num=N;
    fn get(&self)->&Rect<Self::Num>{
        &self.rect
    }
}
/*
impl<N:NumTrait,T:IsPoint<Num=N>> IsPoint for BBox<N,T>{
    type Num=N;
    fn get_center(&self)->[Self::Num;2]{
        self.inner.get_center()
    }
}
*/

pub mod fast_alloc{
    use super::*;
    pub fn new<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(axis:A,n:N,bots:&[T],mut aabb_create:F,ka:K,height:usize,par:JJ)->(DinoTree<A,N,BBox<Num,T>>,K){   
        

        pub struct Cont2<N:NumTrait>{
            rect:Rect<N>,
            pub index:u32
        }
        unsafe impl<N:NumTrait> HasAabb for Cont2<N>{
            type Num=N;
            fn get(&self)->&Rect<N>{
                &self.rect
            }
        }

        let num_bots=bots.len();
        let max=std::u32::MAX;
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let mut conts:Vec<Cont2<Num>>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
        }).collect();
        
    
        let (mut tree2,_bag)=KdTree::new(axis,&mut conts,height,ka,par);
        
        let mover=Mover(tree2.get_tree().create_down().dfs_inorder_iter().flat_map(|(node,_extra)|{
            node.range.iter()
        }).map(|a|a.index).collect());


        let height=tree2.get_tree().get_height();                
        let num_nodes=tree2.get_tree().get_nodes().len();


        let ii=tree2.get_tree_mut().create_down_mut().map(|node,eextra|{
            let l=tree_alloc::NodeConstructor{misc:n,it:node.range.iter_mut().map(|b|{
                BBox{rect:b.rect,inner:bots[b.index as usize]}
            })};

            let extra=match eextra{
                Some(())=>{
                    Some(tree_alloc::ExtraConstructor{
                        comp:Some(node.div)
                    })
                },
                None=>{
                    None
                }
            };

            (l,extra)
        });

        let tree = TreeAllocDstDfsOrder::new(ii,height,num_nodes,num_bots);
        

        let fb=DinoTreeRaw{axis,height,num_nodes,num_bots,alloc:tree};
        let tree=DinoTree{mover,tree:fb};


        //debug_assert!(tree.are_invariants_met().is_ok());
        (tree,_bag)
        
    }
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
/// in user defined algorithms.
///
pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
    mover:Mover,
    tree:DinoTreeRaw<A,N,T>,
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{
    
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{  
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let ka=advanced::SplitterEmpty;

        //on xps13 5 seems good
        const DEPTH_SEQ:usize=2;

        let gg=if height<=DEPTH_SEQ{
            0
        }else{
            height-DEPTH_SEQ
        };
        
        let dlevel=par::Parallel::new(Depth(gg));

        fast_alloc::new(axis,n,bots,aabb_create,ka,height,dlevel).0
    }

    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let ka=advanced::SplitterEmpty;
        fast_alloc::new(axis,n,bots,aabb_create,ka,height,par::Sequential).0
    }

}


impl<A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTree<A,N,T>{
    
    ///Transform the current tree to have a different extra component to each node.
    pub fn with_extra<N2:Copy>(self,n2:N2)->DinoTree<A,N2,T>{
        let (mover,fb)={
            let axis=self.tree.get_axis();
            

            let height=self.height();
            let num_nodes=self.tree.get_num_nodes();
            let num_bots=self.tree.get_num_bots();

            let mover=self.mover.clone();
            let ii=self.vistr().map(|node,eextra|{
                let l=tree_alloc::NodeConstructor{misc:n2,it:node.range.iter().map(|b|*b)};

                let extra=match eextra{
                    Some(extra)=>{
                        Some(tree_alloc::ExtraConstructor{
                            comp:extra.map(|a|*a)
                        })
                    },
                    None=>{
                        None
                    }
                };

                (l,extra)
            });
            
            let tree=TreeAllocDstDfsOrder::new(ii,height,num_nodes,num_bots);
            (mover,DinoTreeRaw{axis,height,num_nodes,num_bots,alloc:tree})
        };

        DinoTree{mover,tree:fb}
    }
}



pub(crate) mod iter_mut{
    use super::*;

    pub fn convert<'a,N:'a,T:HasAabb+'a>(a:(&'a mut NodeDyn<N, T>,Option<Option<&'a FullComp<T::Num>>>))->std::slice::IterMut<'a,T>{
        a.0.range.iter_mut()
    }
    
    pub type FF<'a,N,T>=fn(  (&'a mut NodeDyn<N, T>,Option<Option<&'a FullComp<<T as HasAabb>::Num>>>) ) -> std::slice::IterMut<'a,T>;
    
    ///Iterator over all the elements in the tree in dfs in order- not the original order.
    pub struct TreeIterMut<'a,N:'a,T:HasAabb+'a>{
        pub(crate) length:usize,
        pub(crate) num:usize,
        pub(crate) it:std::iter::FlatMap<
            compt::DfsInOrderIter<VistrMut<'a,N,T>>,
            std::slice::IterMut<'a,T>,
            FF<'a,N,T>
        >
    }
    impl<'a,N,T:HasAabb> Iterator for TreeIterMut<'a,N,T>{
        type Item=&'a mut T;
        fn next(&mut self)->Option<Self::Item>{
            self.num+=1;
            self.it.next()
        }
        fn size_hint(&self)->(usize,Option<usize>){
            (self.length-self.num,Some(self.length-self.num))
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
        pub(crate) length:usize,
        pub(crate) num:usize,
        pub(crate) it:std::iter::FlatMap<
            compt::DfsInOrderIter<Vistr<'a,N,T>>,
            std::slice::Iter<'a,T>,
            FF<'a,N,T>
        >
    }
    impl<'a,N,T:HasAabb> Iterator for TreeIter<'a,N,T>{
        type Item=&'a T;
        fn next(&mut self)->Option<Self::Item>{
            //self.length-=1;
            self.num+=1;
            self.it.next()
        }
        fn size_hint(&self)->(usize,Option<usize>){
            (self.length-self.num,Some(self.length-self.num))
        }
    }

    impl<'a,N,T:HasAabb> std::iter::FusedIterator for TreeIter<'a,N,T>{}
    impl<'a,N,T:HasAabb> std::iter::ExactSizeIterator for TreeIter<'a,N,T>{}
    unsafe impl<'a,N,T:HasAabb> std::iter::TrustedLen for TreeIter<'a,N,T>{}
}



impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{

    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree be copied back into the original list.
    pub fn apply<X>(&mut self,bots:&mut [X],conv:impl Fn(&T,&mut X)){
        
        assert_eq!(bots.len(),self.num_bots());
        let mut counter=0;

        for (bot,mov) in self.iter().zip(self.mover.0.iter()){
            let target=&mut bots[*mov as usize];
            conv(bot,target);
            counter+=1;
        }
        assert_eq!(counter,self.mover.0.len());
    }

    ///Iterate over al the bots in the tree. The order in which they are iterated is dfs in order.
    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    ///But this is useful if you need to iterate over all the bots aabbs.
    pub fn iter_mut<'a>(&'a mut self)->iter_mut::TreeIterMut<'a,N,T>{
        let length=self.tree.get_num_bots();
        let it=self.vistr_mut().dfs_inorder_iter().flat_map(iter_mut::convert as iter_mut::FF<N,T>);
        iter_mut::TreeIterMut{length,it,num:0}
    }

    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter<'a>(&'a self)->iter_const::TreeIter<'a,N,T>{
        //self.get_iter().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter())
        let length=self.tree.get_num_bots();
        let it=self.vistr().dfs_inorder_iter().flat_map(iter_const::convert as iter_const::FF<N,T>);
        iter_const::TreeIter{length,it,num:0}

    }
    
    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    pub fn axis(&self)->A{
        self.tree.get_axis()
    }

    ///Get the height of the tree.
    pub fn height(&self)->usize{
        self.tree.get_height()
    }

    ///Create a mutable tree visitor.
    pub fn vistr_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        self.tree.get_iter_mut()
    }

    ///Create an immutable tree visitor.
    pub fn vistr<'b>(&'b self)->Vistr<'b,N,T>{
        self.tree.get_iter()
    }

    ///Returns the number of bots that are in the tree.
    pub fn num_bots(&self)->usize{
        self.tree.num_bots
    }
    pub fn num_nodes(&self)->usize{
        self.tree.num_nodes
    }
}

//TODO get rid of this layer. It doesnt add anything.
use tree_alloc::TreeAllocDstDfsOrder;

pub struct DinoTreeRaw<A:AxisTrait,N,T:HasAabb>{
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    alloc:TreeAllocDstDfsOrder<N,T>,
    axis:A
}

impl<A:AxisTrait,N,T:HasAabb> DinoTreeRaw<A,N,T>{
   

    pub fn get_axis(&self)->A{
        self.axis
    }
    pub fn get_num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn get_num_bots(&self)->usize{
        self.num_bots
    }
    pub fn get_height(&self)->usize{
        self.height
    }
    pub fn get_iter_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        self.alloc.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->Vistr<'b,N,T>{
        self.alloc.get_iter()
    }
}



#[derive(Clone)]
pub struct Mover(
    pub Vec<u32>
);
