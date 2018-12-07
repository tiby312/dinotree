use inner_prelude::*;



///The common struct between leaf nodes and non leaf nodes.
///It is a dynamically sized type.
pub struct NodeDyn<N,T>{
    ///Some tree query algorithms need memory on a per node basis.
    ///By embedding the memory directly in the tree we gain very good memory locality.
    pub misc:N,
    
    ///The list of bots. Sorted along the alternate axis for that level of the tree.
    pub range:[T]
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



struct NodeDstDyn<N,T:HasAabb>{
    //This term can't live in fullcomp, since every if there are no bots in a node, or below,
    //we might want to traverse the lower children to construct the tree properly.
    next_nodes:[u32;2], //offset from parents in terms of bytes   //TODO change these to i32!!!!!!!!!!!!!

    comp:FullComp<T::Num>,
        
    node:NodeDynWrap<N,T>
}

impl<N,T:HasAabb> NodeDstDyn<N,T>{
    fn as_ptr(&self)->*const u8{
        let alloc::Repr{ptr,size:_size}=unsafe{std::mem::transmute(self)};
        ptr
    }
}

struct NodeDynWrap<N,T>{
    num:u32, //TODO hcange these to i32
    dyn:NodeDyn<N,T>
}

impl<N,T:HasAabb> NodeDynWrap<N,T>{
    fn as_ptr(&self)->*const u8{
        let alloc::Repr{ptr,size:_size}=unsafe{std::mem::transmute(self)};
        ptr
    }
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
    type Item=&'a NodeDyn<N,T>;

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
                        std::mem::transmute(alloc::Repr{ptr:self.ptr,size:0})
                    };
                    let ll=node.node.num as usize;
                    let node:&NodeDstDyn<N,T>={
                        std::mem::transmute(alloc::Repr{ptr:self.ptr,size:ll})
                    };
                    node
                };

                let left_pointer=(self.ptr as *const u8).offset(- (node.next_nodes[0] as isize)) ;
                let right_pointer=(self.ptr as *const u8).offset(node.next_nodes[1] as isize);
                let left_pointer=left_pointer.as_ref().unwrap();
                let right_pointer=right_pointer.as_ref().unwrap();

                let a=Vistr{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=Vistr{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                let comp=if node.node.num==0{
                    None
                }else{
                    Some(&node.comp)
                };
                (&node.node.dyn,Some((comp,a,b)))
            }else{
                let node={
                    let node:&NodeDynWrap<N,T>={
                        std::mem::transmute(alloc::Repr{ptr:self.ptr,size:0})
                    };
                    let ll=node.num as usize;
                    let node:&NodeDynWrap<N,T>={
                        std::mem::transmute(alloc::Repr{ptr:self.ptr,size:ll})
                    };
                    node
                };

                (&node.dyn,None)
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
    type Item=&'a mut NodeDyn<N,T>;

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
                        std::mem::transmute(alloc::ReprMut{ptr:self.ptr,size:0})
                    };
                    let ll=node.node.num as usize;
                    let node:&mut NodeDstDyn<N,T>={
                        std::mem::transmute(alloc::ReprMut{ptr:self.ptr,size:ll})
                    };
                    node
                };

                let left_pointer=(self.ptr as *mut u8).offset(-(node.next_nodes[0] as isize)) ;
                let right_pointer=(self.ptr as *mut u8).offset(node.next_nodes[1] as isize);
                let left_pointer=left_pointer.as_mut().unwrap();
                let right_pointer=right_pointer.as_mut().unwrap();

                let a=VistrMut{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=VistrMut{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                let comp=if node.node.num==0{
                    None
                }else{
                    Some(&node.comp)
                };
                (&mut node.node.dyn,Some((comp,a,b)))
            }else{
                let node={
                    let node:&mut NodeDynWrap<N,T>={
                        std::mem::transmute(alloc::ReprMut{ptr:self.ptr,size:0})
                    };
                    let ll=node.num as usize;
                    let node:&mut NodeDynWrap<N,T>={
                        std::mem::transmute(alloc::ReprMut{ptr:self.ptr,size:ll})
                    };
                    node
                };

                (&mut node.dyn,None)
            }
        }
        
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}




#[repr(C)]
pub struct ReprMut<T>{
    pub ptr:*mut T,
    pub size:usize,
}

#[repr(C)]
pub struct Repr<T>{
    pub ptr:*const T,
    pub size:usize,
}


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






unsafe impl<A:AxisTrait,T:HasAabb,N> Send for TreeInner<A,T,N>{}
unsafe impl<A:AxisTrait,T:HasAabb,N> Sync for TreeInner<A,T,N>{}

pub(crate) struct TreeInner<A:AxisTrait,T:HasAabb,N>{
    axis:A,
    _mem:Vec<u8>,
    root:usize,
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    _p:PhantomData<(T,N)>
}


impl<A:AxisTrait,T:HasAabb,N> TreeInner<A,T,N>{
    pub(crate) fn axis(&self)->A{
        self.axis
    }
    pub(crate) fn height(&self)->usize{
        self.height
    }
    pub(crate) fn num_nodes(&self)->usize{
        self.num_nodes
    }
    pub(crate) fn num_bots(&self)->usize{
        self.num_bots
    }
    pub(crate) fn vistr(&self)->Vistr<N,T>{
        let ptr=unsafe{std::mem::transmute(self.root)};
            
        Vistr::new(ptr,self.height)
        
    }
    pub(crate) fn vistr_mut(&mut self)->VistrMut<N,T>{
        let ptr=unsafe{std::mem::transmute(self.root)};
        VistrMut::new(ptr,self.height)
    }
}


impl<'a,A:AxisTrait,T:HasAabb+Copy+'a,N:Copy+'a> TreeInner<A,T,N>{

    pub(crate) fn from_dfs_in_order2<K:ExactSizeIterator<Item=T>+TrustedLen,I:compt::Visitor<Item=K,NonLeafItem=FullCompOrEmpty<T::Num>>>(axis:A,height:usize,num_bots:usize,a:I,n:N)->TreeInner<A,T,N>{
        
        let num_nodes=1usize.rotate_left(height as u32)-1;
        

        let mut mem=NodeAllocator::new(height,num_bots);
        let root=handle(a,&mut mem,n);
        return TreeInner{axis,_mem:mem.into_inner(),root,height,num_nodes,num_bots,_p:PhantomData};
        

        fn handle<N:Copy,T:HasAabb+Copy,K:ExactSizeIterator<Item=T>+TrustedLen,I:compt::Visitor<Item=K,NonLeafItem=FullCompOrEmpty<T::Num>>>(a:I,na:&mut NodeAllocator<N,T>,n:N)->usize{


            let (nn,rest)=a.next();

            match rest{
                Some((fullcomp,left,right))=>{
                    let left_addr=handle(left,na,n);


                    let node=na.create_non_leaf(fullcomp,n,nn);

                    let right_addr=handle(right,na,n);


                    let ll=(node.inner_ptr() as usize) - (left_addr as usize);
                    let rr=(right_addr as usize) - (node.inner_ptr() as usize);
                    //println!("ll rr={:?}",(ll,rr));
                    let node=node.into_node(ll as u32,rr as u32);
                    node.as_ptr() as usize
                },
                None=>{
                    //println!("leaf!");
                    let node=na.create_leaf(nn,n);
                    node.as_ptr() as usize
                }
            }
        }
    }
    
    /*
    //TODO measure using this
    pub(crate) fn from_dfs_in_order<T2:HasAabb<Num=T::Num>>(axis:A,num_bots:usize,a:&compt::dfs_order::CompleteTree<Node2<T2>>,func: impl Fn(&T2)->T+Copy,n:N)->TreeInner<A,T,N>{

        let num_nodes=a.get_nodes().len();
        let height=a.get_height();
        

        let mut mem=NodeAllocator::new(height,num_bots);
        let root=handle(a.vistr(),&mut mem,func,n);
        return TreeInner{axis,_mem:mem.into_inner(),root,height,num_nodes,num_bots,_p:PhantomData};
        

        fn handle<N:Copy,T:HasAabb+Copy,T2:HasAabb<Num=T::Num>>(v:compt::dfs_order::Vistr<Node2<T2>>,na:&mut NodeAllocator<N,T>,func: impl Fn(&T2)->T+Copy,n:N)->usize{


            let (nn,rest)=v.next();

            match rest{
                Some((_,left,right))=>{
                    let left_addr=handle(left,na,func,n);


                    let node=na.create_non_leaf(nn.fullcomp,n,nn.mid.iter().map(|a|func(a)));

                    let right_addr=handle(right,na,func,n);


                    let ll=(node.inner_ptr() as usize) - (left_addr as usize);
                    let rr=(right_addr as usize) - (node.inner_ptr() as usize);
                    //println!("ll rr={:?}",(ll,rr));
                    let node=node.into_node(ll as u32,rr as u32);
                    node.as_ptr() as usize
                },
                None=>{
                    //println!("leaf!");
                    let node=na.create_leaf(nn.mid.iter().map(|a|func(a)),n);
                    node.as_ptr() as usize
                }
            }
        }
    }
    */
    
}


