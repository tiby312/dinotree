use inner_prelude::*;




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
    axis:A,
    mem:Vec<u8>,
    root:usize,
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    _p:PhantomData<(T,N)>
}


impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{
    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        VistrMut::new(&mut self.mem[self.root],self.height)
    }
    pub fn vistr(&self)->Vistr<N,T>{
        Vistr::new(&self.mem[self.root],self.height)
    }
    pub fn height(&self)->usize{
        self.height
    }
    pub fn num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn axis(&self)->A{
        self.axis
    }
    pub fn num_bots(&self)->usize{
        self.num_bots
    }
    
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{


    pub fn new<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
        rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->(DinoTree<A,N,BBox<Num,T>>,Vec<u32>)
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
                tree::recurse_rebal1(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes
            },
            RebalStrat::Second=>{
                let mut nodes=SmallVec::new();
                tree::recurse_rebal2(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes.into_vec()
            },
            RebalStrat::Third=>{
                let mut nodes=Vec::new();
                tree::recurse_rebal3(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes
            }
        };
        let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();
        let (root,mem)=from_dfs_in_order1(height,num_bots,tree.vistr().map(|item,nonleaf|{
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

        
        //let tree=DinoTree{mover,alloc};



        let num_nodes=1usize.rotate_left(height as u32)-1;
        (DinoTree{axis,mem,root,height,num_nodes,num_bots,_p:PhantomData},mover)
        //tree
        
    }
    /*
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
    */

}


unsafe impl<A:AxisTrait,T:HasAabb+Send,N:Send> Send for DinoTree<A,N,T>{}
unsafe impl<A:AxisTrait,T:HasAabb+Sync,N:Sync> Sync for DinoTree<A,N,T>{}


impl<A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTree<A,N,T>{
    
    ///Transform the current tree to have a different extra component to each node.
    pub fn with_extra<N2:Copy>(self,n2:N2)->DinoTree<A,N2,T>{
        
        let axis=self.axis;
        let height=self.height;
        let num_bots=self.num_bots;//num_bots();
        
        let vistr=self.vistr();//Vistr::<N,T>::new(self.root,height);
        let (root,mem)=from_dfs_in_order1(height,num_bots,vistr.map(|item,nonleaf|{
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

        let num_nodes=1usize.rotate_left(height as u32)-1;
        
        return DinoTree{axis,mem,root,height,num_nodes,num_bots,_p:PhantomData};
    }

}



fn from_dfs_in_order1<K:ExactSizeIterator<Item=T>+TrustedLen,I:compt::Visitor<Item=K,NonLeafItem=FullCompOrEmpty<T::Num>>,T:HasAabb+Copy,N:Copy>(
    height:usize,num_bots:usize,a:I,n:N)->(usize,Vec<u8>){

    use self::nodealloc::*;

    

    let mut mem=NodeAllocator2::new(height,num_bots);
    let root=handle(a,&mut mem,n);

    return(root.0,mem.into_inner());
    //return TreeInner{axis,mem:mem.into_inner(),root:root.0,height,num_nodes,num_bots,_p:PhantomData};
    

    fn handle<N:Copy,T:HasAabb+Copy,K:ExactSizeIterator<Item=T>+TrustedLen,I:compt::Visitor<Item=K,NonLeafItem=FullCompOrEmpty<T::Num>>>(a:I,na:&mut NodeAllocator2<N,T>,n:N)->BufferIndex{


        let (nn,rest)=a.next();

        match rest{
            Some((fullcomp,left,right))=>{
                let left_index=handle(left,na,n);

                let node=na.push_non_leaf(fullcomp,n,nn);

                let right_index=handle(right,na,n);

                na.connect_children_nodes(node,left_index,right_index)
            },
            None=>{
                na.push_leaf(nn,n)
            }
        }
    }
}








///The common struct between leaf nodes and non leaf nodes.
///It is a dynamically sized type.

pub struct NodeDyn<N,T>{
    ///Some tree query algorithms need memory on a per node basis.
    ///By embedding the memory directly in the tree we gain very good memory locality.
    pub misc:N,
    
    ///The list of bots. Sorted along the alternate axis for that level of the tree.
    pub range:[T]
}


struct NodeDstDyn<N,T:HasAabb>{
    //This term can't live in fullcomp, since every if there are no bots in a node, or below,
    //we might want to traverse the lower children to construct the tree properly.
    next_nodes:[u32;2], //offset from parents in terms of bytes   //TODO change these to i32!!!!!!!!!!!!!

    comp:FullComp<T::Num>,
        
    node:NodeDynWrap<N,T>,

}



struct NodeDynWrap<N,T>{
    num:u32, //TODO hcange these to i32
    dyn:NodeDyn<N,T>
}






/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct Vistr<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a (T,N)>
}

impl<'a,N:'a,T:HasAabb+'a> Vistr<'a,N,T>{
    fn new(root:&'a u8,height:usize)->Vistr<'a,N,T>{
        Vistr{ptr:root,height,depth:0,_p:PhantomData}
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->Vistr<'b,N,T>{
        //Vistr{inner:self.inner.create_wrap()}
        Vistr{ptr:self.ptr,height:self.height,depth:self.depth,_p:PhantomData}   
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for Vistr<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Visitor for Vistr<'a,N,T>{
    type Item=NodeRef<'a,N,T>;

    ///An option of the NonLeafItem is returned to indicate that this node
    ///and all leaves under this node do not have any bots. If such cases,
    ///it does not make sense to have a divider since there is no median to use
    ///to make it.
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                
                let node={
                    let node:&NodeDstDyn<N,T>={
                        std::mem::transmute(tools::Repr{ptr:self.ptr,size:0})
                    };
                    let ll=node.node.num as usize;
                    let node:&NodeDstDyn<N,T>={
                        std::mem::transmute(tools::Repr{ptr:self.ptr,size:ll})
                    };
                    node
                };

                let left_pointer=(self.ptr as *const u8).wrapping_sub(node.next_nodes[0] as usize);
                let right_pointer=(self.ptr as *const u8).wrapping_add(node.next_nodes[1] as usize);
                let left_pointer=left_pointer.as_ref().unwrap();
                let right_pointer=right_pointer.as_ref().unwrap();

                let a=Vistr{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=Vistr{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                let comp=if node.node.num==0{
                    None
                }else{
                    Some(&node.comp)
                };

                let n=NodeRef{misc:&node.node.dyn.misc,range:&node.node.dyn.range};
                (n,Some((comp,a,b)))
            }else{
                let node={
                    let node:&NodeDynWrap<N,T>={
                        std::mem::transmute(tools::Repr{ptr:self.ptr,size:0})
                    };
                    let ll=node.num as usize;
                    let node:&NodeDynWrap<N,T>={
                        std::mem::transmute(tools::Repr{ptr:self.ptr,size:ll})
                    };
                    node
                };
                
                let n=NodeRef{misc:&node.dyn.misc,range:&node.dyn.range};
                (n,None)
            }
        }
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}



/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct VistrMut<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a mut u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a mut (T,N)>
}

impl<'a,N:'a,T:HasAabb+'a> VistrMut<'a,N,T>{
    fn new(root:&'a mut u8,height:usize)->VistrMut<'a,N,T>{
        VistrMut{ptr:root,height,depth:0,_p:PhantomData}
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        VistrMut{ptr:self.ptr,height:self.height,depth:self.depth,_p:PhantomData}   
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for VistrMut<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Visitor for VistrMut<'a,N,T>{
    type Item=NodeRefMut<'a,N,T>;

    ///An option of the NonLeafItem is returned to indicate that this node
    ///and all leaves under this node do not have any bots. If such cases,
    ///it does not make sense to have a divider since there is no median to use
    ///to make it.
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                
                let node={
                    let node:&mut NodeDstDyn<N,T>={
                        std::mem::transmute(tools::ReprMut{ptr:self.ptr,size:0})
                    };
                    let ll=node.node.num as usize;
                    let node:&mut NodeDstDyn<N,T>={
                        std::mem::transmute(tools::ReprMut{ptr:self.ptr,size:ll})
                    };
                    node
                };

                let left_pointer=(self.ptr as *mut u8).wrapping_sub(node.next_nodes[0] as usize);
                let right_pointer=(self.ptr as *mut u8).wrapping_add(node.next_nodes[1] as usize);
                let left_pointer=left_pointer.as_mut().unwrap();
                let right_pointer=right_pointer.as_mut().unwrap();

                let a=VistrMut{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=VistrMut{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                let comp=if node.node.num==0{
                    None
                }else{
                    Some(&node.comp)
                };

                let n=NodeRefMut{misc:&mut node.node.dyn.misc,range:&mut node.node.dyn.range};
                
                (n,Some((comp,a,b)))
            }else{
                let node={
                    let node:&mut NodeDynWrap<N,T>={
                        std::mem::transmute(tools::ReprMut{ptr:self.ptr,size:0})
                    };
                    let ll=node.num as usize;
                    let node:&mut NodeDynWrap<N,T>={
                        std::mem::transmute(tools::ReprMut{ptr:self.ptr,size:ll})
                    };
                    node
                };

                let n=NodeRefMut{misc:&mut node.dyn.misc,range:&mut node.dyn.range};
                (n,None)
            }
        }
        
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}




mod nodealloc{
    use super::*;
    #[derive(Copy,Clone,Debug)]
    pub struct BufferIndex(pub usize);

    pub struct NodeLeafReserve2{
        node:BufferIndex
    }

    pub struct NodeAllocator2<N,T:HasAabb>{
        mem:Vec<u8>,
        _p:PhantomData<(N,T)>
    }

    impl<N,T:HasAabb> NodeAllocator2<N,T>{
        pub fn new(height:usize,num_bots:usize)->NodeAllocator2<N,T>{

            fn calculate_space_needed<N,T:HasAabb>(depth:usize,height:usize,num_bots:usize)->usize{
                use std::mem::*;
                let number_of_levels_left=height-depth;

                let num_nodes_left=1usize.rotate_left(number_of_levels_left as u32)-1;

                let val1:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(tools::ReprMut{ptr:0x128 as *mut u8,size:0})};
                let val2:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(tools::ReprMut{ptr:0x128 as *mut u8,size:0})};
                 
                let num_non_leafs=num_nodes_left/2;
                let num_leafs=num_nodes_left-num_non_leafs;

                let k=(num_bots)*(std::mem::size_of::<T>())+
                size_of_val(val1)*num_non_leafs+
                size_of_val(val2)*num_leafs+
                align_of_val(val1)*(1+(num_non_leafs/2))+
                align_of_val(val2)*(1+(num_leafs/2));
                k
            }



            let start_cap=calculate_space_needed::<N,T>(0,height,num_bots);
           

            NodeAllocator2{mem:Vec::with_capacity(start_cap),_p:PhantomData}
        }

        pub fn into_inner(self)->Vec<u8>{
            //println!("wasted space={:?}",self.mem.capacity()-self.mem.len());
            //self.mem.shrink_to_fit();
            self.mem
        }
        pub fn connect_children_nodes(&mut self,a:NodeLeafReserve2,left:BufferIndex,right:BufferIndex)->BufferIndex{
            unsafe{
                let val:&mut NodeDstDyn<N,T>=std::mem::transmute(tools::ReprMut{ptr:&mut self.mem[a.node.0] as *mut u8,size:0});
                

                assert!(a.node.0>left.0);
                assert!(right.0>a.node.0);
                let ll=a.node.0-left.0;
                let rr=right.0-a.node.0;

                val.next_nodes=[ll as u32,rr as u32];
                a.node
            }
        }
        pub fn push_non_leaf(&mut self,fullcomp:FullCompOrEmpty<T::Num>,n:N,bots:impl ExactSizeIterator<Item=T> + TrustedLen)->NodeLeafReserve2{
            let (index,node2)=unsafe{
                let (align,siz)={
                    let val:&mut NodeDstDyn<N,T>=std::mem::transmute(tools::ReprMut{ptr:0x128 as *mut u8,size:bots.len()});
                    (align_of_val(val),size_of_val(val))
                };

                let l=self.mem.len();
                let off=self.mem[l..].as_ptr().align_offset(align);
                
                self.mem.resize_with(l+off,||std::mem::uninitialized());

                

                let l=self.mem.len();
                assert_eq!(self.mem[l..].as_ptr().align_offset(align),0);
                self.mem.resize_with(l+siz,||std::mem::uninitialized());
                let node:&mut NodeDstDyn<N,T>=std::mem::transmute(tools::ReprMut{ptr:self.mem[l..].as_mut_ptr(),size:bots.len()});
                (l,node)
            };


            
            let fullcomp=match fullcomp{
                FullCompOrEmpty::NonEmpty(fullcomp)=>{
                    fullcomp
                },
                FullCompOrEmpty::Empty()=>{
                    unsafe{
                        let mut fullcomp=std::mem::uninitialized();
                        std::ptr::write_bytes(&mut fullcomp,0,std::mem::size_of::<FullComp<T::Num>>());
                        fullcomp
                    }
                }
            };
            

            node2.comp=fullcomp;
            node2.node.num=bots.len() as u32;
            node2.node.dyn.misc=n;
            for (a,b) in node2.node.dyn.range.iter_mut().zip(bots){
                *a=b;
            }

            NodeLeafReserve2{node:BufferIndex(index)}
        }

        pub fn push_leaf(&mut self,bots:impl ExactSizeIterator<Item=T> + TrustedLen,n:N)->BufferIndex{
            //assert!(node.fullcomp.is_none());
            let (index,node2)=unsafe{
                let (align,siz)={
                    let val:&mut NodeDynWrap<N,T>=std::mem::transmute(tools::ReprMut{ptr:0x128 as *mut u8,size:bots.len()});
                    (align_of_val(val),size_of_val(val))
                };

                let l=self.mem.len();
                let off=self.mem[l..].as_ptr().align_offset(align);
                self.mem.resize_with(l+off,||std::mem::uninitialized());


                let l=self.mem.len();
                self.mem.resize_with(l+siz,||std::mem::uninitialized());
                let node:&mut NodeDynWrap<N,T>=std::mem::transmute(tools::ReprMut{ptr:self.mem[l..].as_mut_ptr(),size:bots.len()});

                //assert_eq!(self.mem[ll..].as_ptr(),self.mem[l..].as_ptr().wrapping_add(siz));
                (l,node)
            };

            node2.num=bots.len() as u32;
            node2.dyn.misc=n;
            for(a,b) in node2.dyn.range.iter_mut().zip(bots){
                *a=b;
            }
            BufferIndex(index)
        }
    }
}



/*


struct NodeLeafReserve<'a,N,T:HasAabb>{
    node:&'a mut NodeDstDyn<N,T>
}
impl<'a,N,T:HasAabb> NodeLeafReserve<'a,N,T>{
    fn inner_ptr(&self)->*const u8{
        self.node.as_ptr()
    }
    fn into_node(self,left:u32,right:u32)->&'a mut NodeDstDyn<N,T>{
        self.node.next_nodes=[left,right];
        self.node
    }
}

struct NodeAllocator<'a,N,T>{
    mem:Vec<u8>,
    start_cap:usize,
    _p:PhantomData<&'a mut (N,T)>
}
impl<'a,N,T:HasAabb+Copy> NodeAllocator<'a,N,T>{
    fn new(height:usize,num_bots:usize)->NodeAllocator<'a,N,T>{
        
        fn calculate_space_needed<N,T:HasAabb>(depth:usize,height:usize,num_bots:usize)->usize{
            use std::mem::*;
            let number_of_levels_left=height-depth;

            let num_nodes_left=1usize.rotate_left(number_of_levels_left as u32)-1;

            let val1:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:0})};
            let val2:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:0})};
             
            let num_non_leafs=num_nodes_left/2;
            let num_leafs=num_nodes_left-num_non_leafs;

            let k=(num_bots)*(std::mem::size_of::<T>())+
            (size_of_val(val1)+align_of_val(val1))*num_non_leafs+
            (size_of_val(val2)+align_of_val(val2))*num_leafs;
            k
        }



        let start_cap=calculate_space_needed::<N,T>(0,height,num_bots);
        NodeAllocator{
            mem:Vec::with_capacity(start_cap),start_cap,_p:PhantomData
        }
    }
    fn into_inner(self)->Vec<u8>{
        assert_eq!(self.mem.capacity(),self.start_cap);
        println!("wasted space in bytes={:?}",self.start_cap-self.mem.len());
        self.mem
    }

    fn create_non_leaf(&mut self,fullcomp:FullCompOrEmpty<T::Num>,n:N,bots:impl ExactSizeIterator<Item=T> + TrustedLen)->NodeLeafReserve<'a,N,T>{
        let node2=unsafe{
            let (align,siz)={
                let val:&mut NodeDstDyn<N,T>=std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:bots.len()});
                (align_of_val(val),size_of_val(val))
            };

            let l=self.mem.len();
            let off=self.mem[l..].as_ptr().align_offset(align);
            self.mem.resize_with(l+off,||std::mem::uninitialized());


            let l=self.mem.len();
            self.mem.resize_with(l+siz,||std::mem::uninitialized());
            let node:&'a mut NodeDstDyn<N,T>=std::mem::transmute(ReprMut{ptr:self.mem[l..].as_mut_ptr(),size:bots.len()});
            node
        };

        
        let fullcomp=match fullcomp{
            FullCompOrEmpty::NonEmpty(fullcomp)=>{
                fullcomp
            },
            FullCompOrEmpty::Empty()=>{
                unsafe{
                    let mut fullcomp=std::mem::uninitialized();
                    std::ptr::write_bytes(&mut fullcomp,0,std::mem::size_of::<FullComp<T::Num>>());
                    fullcomp
                }
            }
        };
        

        node2.comp=fullcomp;
        node2.node.num=bots.len() as u32;
        node2.node.dyn.misc=n;
        for (a,b) in node2.node.dyn.range.iter_mut().zip(bots){
            *a=b;
        }
        //node2.node.dyn.range.copy_from_slice(&node.mid);
        NodeLeafReserve{node:node2}
    }
    fn create_leaf(&mut self,bots:impl ExactSizeIterator<Item=T> + TrustedLen,n:N)->&'a mut NodeDynWrap<N,T>{
        //assert!(node.fullcomp.is_none());
        let node2=unsafe{
            let (align,siz)={
                let val:&mut NodeDynWrap<N,T>=std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:bots.len()});
                (align_of_val(val),size_of_val(val))
            };

            let l=self.mem.len();
            let off=self.mem[l..].as_ptr().align_offset(align);
            self.mem.resize_with(l+off,||std::mem::uninitialized());


            let l=self.mem.len();
            self.mem.resize_with(l+siz,||std::mem::uninitialized());
            let node:&'a mut NodeDynWrap<N,T>=std::mem::transmute(ReprMut{ptr:self.mem[l..].as_mut_ptr(),size:bots.len()});
            node
        };

        node2.num=bots.len() as u32;
        node2.dyn.misc=n;
        for(a,b) in node2.dyn.range.iter_mut().zip(bots){
            *a=b;
        }
        node2
    }
}
*/



