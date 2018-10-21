use inner_prelude::*;
use base_kdtree::KdTree;
use HasAabb;
use tree_alloc::NdIterMut;
use tree_alloc::NdIter;
use compt::CTreeIterator;
use axgeom::*;
//use TreeHeightHeur;

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
use std::error::Error;
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

impl<N:NumTrait,T:IsPoint<Num=N>> IsPoint for BBox<N,T>{
    type Num=N;
    fn get_center(&self)->[Self::Num;2]{
        self.inner.get_center()
    }
}

mod fast_alloc{
    use super::*;
    pub fn new<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(axis:A,n:N,bots:&[T],mut aabb_create:F,ka:K,height:usize,par:JJ)->(DynTree<A,N,BBox<Num,T>>,K){   
        

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
        
        let mover=Mover(tree2.get_tree().create_down().dfs_preorder_iter().flat_map(|(node,_extra)|{
            node.range.iter()
        }).map(|a|a.index).collect());


        let height=tree2.get_tree().get_height();                
        let num_nodes=tree2.get_tree().get_nodes().len();


        let ii=tree2.get_tree_mut().create_down_mut().map(|node,eextra|{
            let l=tree_alloc::LeafConstructor{misc:n,it:node.range.iter_mut().map(|b|{
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
        

        let fb=DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree};
        let tree=DynTree{mover,tree:fb};


        //debug_assert!(tree.are_invariants_met().is_ok());
        (tree,_bag)
        
    }
}





/// The tree this crate revoles around.
///
/// The user supplies a list of objects to insert along with a way to create a bounding box for each object.
/// Then the tree is constructed. 
///
/// The bots are unsafely copied into a tree, and then usafely copied back out. The algorithm ensures
/// That even though the ordering is different, this is a bijection between the two sets.
/// So we can safely hide this unsafety from the user.
/// The bots are copied back in the trees drop() method. If the user panics inside of a callback function,
/// The changes to the bots up until that more during the traversal of the tree will take effect when the 
/// trees drop() occurrs.
///
/// Unsafety is used to construct the special variable node size tree structure that is populated with dsts.
///
/// The mutable reference to each element in the callback functions do not point to elements
/// in the user supplied slice of elements. The elements are internally unsafely copied directly into a tree structure
/// and then unsafely copied back into the slice at the end. So do not try storing the mutable references as pointers
/// in the callback functions since they would point to unallocated memory once the tree is destroyed.
///
pub struct DynTree<A:AxisTrait,N,T:HasAabb>{
    mover:Mover,
    tree:DynTreeRaw<A,N,T>,
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DynTree<A,N,BBox<Num,T>>{
    


    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{  
        let height=compute_tree_height_heuristic(bots.len()); 
        let ka=SplitterEmpty;


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



    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{   
        let height=compute_tree_height_heuristic(bots.len()); 
        let ka=SplitterEmpty;
        fast_alloc::new(axis,n,bots,aabb_create,ka,height,par::Sequential).0
    }

    pub fn new_adv<K:Splitter+Send>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:usize,splitter:K,height_switch_seq:usize)->(DynTree<A,N,BBox<Num,T>>,K){   
        //let height=heur.compute_tree_height_heuristic(bots.len()); 
        //let ka=TreeTimer2::new(height);


        //on xps13 5 seems good
        //const DEPTH_SEQ:usize=6;

        let gg=if height<=height_switch_seq{
            0
        }else{
            height-height_switch_seq
        };
        
        let dlevel=par::Parallel::new(Depth(gg));


        let a=fast_alloc::new(axis,n,bots,aabb_create,splitter,height,dlevel);
        a
        //(a.0,(a.1).into_iter())
    }

    pub fn new_adv_seq<K:Splitter>(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>,height:usize,splitter:K)->(DynTree<A,N,BBox<Num,T>>,K){   

        pub struct SplitterWrapper<T>(
            pub T,
        );

        impl<T:Splitter> Splitter for SplitterWrapper<T>{
            fn div(self)->(Self,Self){
                let (a,b)=self.0.div();
                (SplitterWrapper(a),SplitterWrapper(b))
            }
            fn add(self,a:Self)->Self{
                let a=self.0.add(a.0);
                SplitterWrapper(a)
            }
            fn node_start(&mut self){self.0.node_start()}
            fn node_end(&mut self){self.0.node_end()}
        }        
        unsafe impl<T> Send for SplitterWrapper<T>{}
        unsafe impl<T> Sync for SplitterWrapper<T>{}

    
        let (a,b)=fast_alloc::new(axis,n,bots,aabb_create,SplitterWrapper(splitter),height,par::Sequential);
        (a,b.0)
    }
}


impl<A:AxisTrait,N:Copy,T:HasAabb+Copy> DynTree<A,N,T>{
    

    ///Returns the bots to their original ordering.
    pub fn apply_orig_order<X>(&mut self,bots:&mut [X],conv:impl Fn(&T,&mut X)){
        
        assert_eq!(bots.len(),self.get_num_bots());
        let mut counter=0;

        for (bot,mov) in self.tree.get_iter().dfs_preorder_iter().flat_map(|(node,_)|{
            node.range.iter()
        }).zip(self.mover.0.iter()){
            let target=&mut bots[*mov as usize];
            conv(bot,target);
            counter+=1;
        }
        assert_eq!(counter,self.mover.0.len());
    }

    ///Transform the current tree to have a different extra component to each node.
    pub fn with_extra<N2:Copy>(self,n2:N2)->DynTree<A,N2,T>{
        let (mover,fb)={
            let axis=self.tree.get_axis();
            

            let height=self.get_height();
            let num_nodes=self.tree.get_num_nodes();
            let num_bots=self.tree.get_num_bots();

            let mover=self.mover.clone();
            let ii=self.get_iter().map(|node,eextra|{
                let l=tree_alloc::LeafConstructor{misc:n2,it:node.range.iter().map(|b|*b)};

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
            (mover,DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree})
        };

        DynTree{mover,tree:fb}
    }
}


impl<A:AxisTrait,N,T:HasAabb> DynTree<A,N,T> where T::Num : std::fmt::Debug{
    ///Returns Ok, then this tree's invariants are being met.
    ///Should always return true, unless the user corrupts the trees memory
    ///or if the contract of the HasAabb trait are not upheld.
    pub fn are_invariants_met(&self)->Result<(),()>{
        assert_invariants::are_invariants_met(self)
    }
}

impl<A:AxisTrait,N,T:HasAabb> DynTree<A,N,T>{

    ///Iterate over al the bots in the tree. The order in which they are iterated is not important.
    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot_mut<'a>(&'a mut self)->impl Iterator<Item=&'a mut T>{
        self.get_iter_mut().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter_mut())
    }

    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot<'a>(&'a self)->impl Iterator<Item=&'a T>{
        self.get_iter().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter())
    }
    
    
    ///Return the fraction of bots that are in each level of the tree.
    ///The first element is the number of bots in the root level.
    ///The last number is the fraction in the lowest level of the tree.
    ///Ideally the fraction of bots in the lower level of the tree is high.
    pub fn compute_tree_health(&self)->tree_health::LevelRatioIterator<N,T>{
        tree_health::compute_tree_health(self)
    }


    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    pub fn get_axis(&self)->A{
        self.tree.get_axis()
    }

    ///Get the height of the tree.
    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    ///Create a mutable tree visitor.
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.tree.get_iter_mut()
    }

    ///Create an immutable tree visitor.
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.tree.get_iter()
    }


    ///Returns the number of bots that are in the tree.
    pub fn get_num_bots(&self)->usize{
        self.tree.num_bots
    }
    pub fn get_num_nodes(&self)->usize{
        self.tree.num_nodes
    }
}

//TODO get rid of this layer. It doesnt add anything.
use tree_alloc::TreeAllocDstDfsOrder;

pub struct DynTreeRaw<A:AxisTrait,N,T:HasAabb>{
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    alloc:TreeAllocDstDfsOrder<N,T>,
    axis:A
}

impl<A:AxisTrait,N,T:HasAabb> DynTreeRaw<A,N,T>{
   

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
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.alloc.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.alloc.get_iter()
    }
}



#[derive(Clone)]
pub struct Mover(
    pub Vec<u32>
);
