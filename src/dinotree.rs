use inner_prelude::*;
//use dinotree_inner::DinoTreeInner;
use HasAabb;
use tree_alloc::VistrMut;
use tree_alloc::Vistr;
use compt::Visitor;
use advanced::Splitter;
use dinotree_inner::Sorter;
use dinotree_inner::DefaultSorter;

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


#[derive(Debug)]
pub enum RebalStrat{
    First,
    Second,
    Third
}


/*
pub trait RebalStrat{
    fn is_first_strat(&self)->bool;
}
pub struct RebalStrat1;
impl RebalStrat for RebalStrat1{
    fn is_first_strat(&self)->bool{true}
}
pub struct RebalStrat2;
impl RebalStrat for RebalStrat2{
    fn is_first_strat(&self)->bool{false}
}
*/




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


mod double_vec{
    use std::mem::*;
    use std::marker::*;
            
    pub struct DoubleVec{
        v:Vec<u8>,
        start_node:usize,
        start_bot:usize,
        num_nodes:usize,
        num_bots:usize,
        node_size:usize,
        bot_size:usize,
        node_align:usize,
        bot_align:usize,
    }
    impl DoubleVec{
        pub fn new<N,T>(num_nodes:usize,num_bots:usize)->DoubleVec{
            unsafe{
                let node_size=num_nodes*size_of::<N>();
                let bot_size=num_bots*size_of::<T>();

                let node_align=align_of::<N>();
                let bot_align=align_of::<T>();


                let num_bytes=node_align+node_size+bot_align+bot_size;
                let v=Vec::with_capacity(num_bytes);
                let start=v.as_ptr() as usize;
                let end=v.as_ptr().offset(num_bytes as isize) as usize;

                let start_node=v.as_ptr().offset(v.as_ptr().align_offset(node_align) as isize);


                let start_bot=start_node.offset(node_size as isize);
                let start_bot=start_bot.offset(start_bot.align_offset(bot_align) as isize);

                let start_node=start_node as usize;
                let start_bot=start_bot as usize;


                let mut v=DoubleVec{v,start_node,start_bot,num_nodes,num_bots,node_align,bot_align,node_size:size_of::<N>(),bot_size:size_of::<T>()};

                {            
                    let (l,r)=v.get::<N,T>();
                    assert!(l.as_ptr() as usize>=start);
                    assert_eq!((l.as_ptr() as *const u8).align_offset(node_align),0);
                    assert_eq!((r.as_ptr() as *const u8).align_offset(bot_align),0);
                    assert!(l[num_nodes..].as_ptr() as usize <= r.as_ptr() as usize);
                    assert!((r[num_bots..].as_ptr() as usize) < end);
                }

                v
                //assert!(r[num_bots].as_ptr()<start_node.)
            }
        }
        pub fn get<'a,N,T>(&'a mut self)->(&'a mut [N],&'a mut [T]){
            assert_eq!(size_of::<N>(),self.node_size);
            assert_eq!(size_of::<T>(),self.bot_size);
            assert_eq!(align_of::<N>(),self.node_align);
            assert_eq!(align_of::<T>(),self.bot_align);
            unsafe{
                let k1=std::slice::from_raw_parts_mut(self.start_node as *mut N,self.num_nodes);
                let k2=std::slice::from_raw_parts_mut(self.start_bot as *mut T,self.num_bots);
                (k1,k2)
            }
        }
    }
}

pub fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,_ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree<A,N,BBox<Num,T>>
    {   
     


    let num_bots=bots.len();
    let max=std::u32::MAX;
    assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


    let conts=bots.iter().enumerate().map(|(index,k)|{
        Cont2{rect:aabb_create(k),index:index as u32}
                });
    

    //println!("rebal={:?}",rebal_type);

    let (alloc,mover)=match rebal_type{
        RebalStrat::First=>{

            //println!("default");
            let mut conts:Vec<_>=conts.collect();
            
            let mut tree2=compt::dfs_order::CompleteTree::from_dfs_inorder(&mut ||{
                let mid=&mut [];
                //Get rid of zero initialization???
                let fullcomp=unsafe{std::mem::uninitialized()};
                dinotree_inner::Node2{fullcomp,mid}
                
            },height);
            
            {
                let j=tree2.vistr_mut().with_depth(Depth(0));
                dinotree_inner::recurse_rebal(axis,par,&mut conts,j,sorter,&mut advanced::SplitterEmpty);
            }
            let alloc=tree_alloc::TreeInner::from_vistr(num_bots,height,tree2.vistr().map(|item,nonleaf|{
                let x=(n,item.mid.iter().map(|a|BBox{rect:a.rect,inner:bots[a.index as usize]}));
                let fullcomp=match nonleaf{
                    Some(())=>{
                        Some(item.fullcomp)
                    },
                    None=>{
                        None
                    }
                };
                (x,fullcomp)
            }));


            
            let mover={
                let mut mover=Vec::with_capacity(num_bots);
                for (node,_) in tree2.vistr_mut().dfs_inorder_iter(){
                    mover.extend(node.mid.iter().map(|a|a.index));
                }
                mover
            };
            
            (alloc,mover)

        },
        RebalStrat::Second=>{

            //println!("alt");
            use dinotree_inner::Node2;
            let num_nodes=1usize.rotate_left(height as u32)-1;
            
            let mut doublev=double_vec::DoubleVec::new::<Node2<Cont2<Num>>,Cont2<Num>>(num_nodes,num_bots);
            let (nodes,new_bots)=doublev.get::<Node2<Cont2<Num>>,Cont2<Num>>();
            

            for (a ,b) in new_bots.iter_mut().zip_eq(conts){
                *a=b;
            }
            
            
            for n in nodes.iter_mut(){
                n.mid=&mut [];
            }

            let mut tree2=compt::bfs_order_slice::CompleteTree::from_slice(nodes,height).unwrap();

            {
                let j=tree2.vistr_mut().with_depth(Depth(0));
                dinotree_inner::recurse_rebal2(axis,par,new_bots,j,sorter,&mut advanced::SplitterEmpty);
            }

            let alloc=tree_alloc::TreeInner::from_vistr(num_bots,height,tree2.vistr().map(|item,nonleaf|{
                let x=(n,item.mid.iter().map(|a|BBox{rect:a.rect,inner:bots[a.index as usize]}));
                let fullcomp=match nonleaf{
                    Some(())=>{
                        Some(item.fullcomp)
                    },
                    None=>{
                        None
                    }
                };
                (x,fullcomp)
            }));


            //TODO reuse space from double vec!!!!!.???
            let mover={
                let mut mover=Vec::with_capacity(num_bots);
                for (node,_) in tree2.vistr_mut().dfs_inorder_iter(){
                    mover.extend(node.mid.iter().map(|a|a.index));
                }
                mover
            };
            
            (alloc,mover)
        },
        RebalStrat::Third=>{
            //println!("tree");
            let mut tree2=tree_alloc::TreeInner::new(par,sorter,axis,conts,(),height);
        
            let mut alloc:tree_alloc::TreeInner<BBox<Num,T>,N>=tree_alloc::TreeInner::from_vistr(num_bots,height,tree2.vistr().map(|item,nonleaf|{
                let x=(n,item.range.iter().map(|a|{BBox{rect:a.rect,inner:bots[a.index as usize]}}));
                let fullcomp=match nonleaf{
                    Some(fullcomp)=>{
                        match fullcomp{
                            Some(fullcomp)=>Some(*fullcomp),
                            None=>{
                                Some(unsafe{std::mem::uninitialized()})
                            }
                        }
                    },
                    None=>{
                        None
                    }
                };
                (x,fullcomp)
            }));

            //let mover=tree2.into_index_only();
            
            let mover={
                let mut mover=Vec::with_capacity(num_bots);
                for (node,_) in tree2.vistr_mut().dfs_inorder_iter(){
                    mover.extend(node.range.iter().map(|a|a.index));
                }
                mover
            };
            
            (alloc,mover)
        }
    };

    
    let tree=DinoTree{mover,alloc,axis};

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
    alloc:tree_alloc::TreeInner<T,N>,
    axis:A
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
        
        let alloc=tree_alloc::TreeInner::from_vistr(num_bots,height,self.alloc.vistr().map(|item,nonleaf|{
            let x=(n2,item.range.iter().map(|a|*a));
            let fullcomp=match nonleaf{
                Some(fullcomp)=>{
                    match fullcomp{
                        Some(fullcomp)=>Some(*fullcomp),
                        None=>{
                            Some(unsafe{std::mem::uninitialized()})
                        }
                    }
                },
                None=>{
                    None
                }
            };
            (x,fullcomp)
        }));



        DinoTree{mover,axis,alloc}
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
        self.axis
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
