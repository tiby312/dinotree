use inner_prelude::*;
use advanced::Splitter;
use tree_alloc::*;
use std::marker::PhantomData;
use dinotree_inner::*;
use std::iter::TrustedLen;

use std::mem::*;

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

pub struct NodeLeafReserve<'a,N,T:HasAabb>{
    node:&'a mut NodeDstDyn<N,T>
}
impl<'a,N,T:HasAabb> NodeLeafReserve<'a,N,T>{
    pub fn inner_ptr(&self)->*const u8{
        self.node.as_ptr()
    }
    pub fn into_node(self,left:i32,right:i32)->&'a mut NodeDstDyn<N,T>{
        self.node.next_nodes=[left,right];
        self.node
    }
}



pub struct NodeAllocator<'a,N,T>{
    mem:Vec<u8>,
    _p:PhantomData<&'a mut (N,T)>
}
impl<'a,N,T:HasAabb+Copy> NodeAllocator<'a,N,T>{
    pub fn new()->NodeAllocator<'a,N,T>{
        
        NodeAllocator{
            mem:Vec::new(),_p:PhantomData
        }
    }
    pub fn into_inner(self)->Vec<u8>{
        self.mem
    }
    pub fn as_ptr(&self)->*const u8{
        self.mem.as_ptr()
    }

    pub fn create_non_leaf(&mut self,fullcomp:Option<FullComp<T::Num>>,n:N,bots:impl ExactSizeIterator<Item=T> + TrustedLen)->NodeLeafReserve<'a,N,T>{
        let node2=unsafe{
            let (align,siz)={
                let val:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:bots.len()})};
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
            Some(fullcomp)=>{
                fullcomp
            },
            None=>{
                unsafe{
                    let mut fullcomp=std::mem::uninitialized();
                    std::ptr::write_bytes(&mut fullcomp,0,std::mem::size_of::<FullComp<T::Num>>());
                    fullcomp
                }
            }
        };

        node2.comp=fullcomp;
        node2.node.num=bots.len() as i32;
        node2.node.dyn.misc=n;
        for (a,b) in node2.node.dyn.range.iter_mut().zip(bots){
            *a=b;
        }
        //node2.node.dyn.range.copy_from_slice(&node.mid);
        NodeLeafReserve{node:node2}
    }
    pub fn create_leaf(&mut self,bots:impl ExactSizeIterator<Item=T> + TrustedLen,n:N)->&'a mut NodeDynWrap<N,T>{
        //assert!(node.fullcomp.is_none());
        let node2=unsafe{
            let (align,siz)={
                let val:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:bots.len()})};
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

        node2.num=bots.len() as i32;
        node2.dyn.misc=n;
        for(a,b) in node2.dyn.range.iter_mut().zip(bots){
            *a=b;
        }
        //node2.dyn.range.copy_from_slice(&node.mid);
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
    pub fn axis(&self)->A{
        self.axis
    }
    pub fn height(&self)->usize{
        self.height
    }
    pub fn num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn num_bots(&self)->usize{
        self.num_bots
    }
    pub fn vistr(&self)->Vistr<N,T>{
        let ptr=unsafe{std::mem::transmute(self.root)};
            
        Vistr::new(ptr,self.height)
        
    }
    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        let ptr=unsafe{std::mem::transmute(self.root)};
        VistrMut::new(ptr,self.height)
    }
}


impl<'a,A:AxisTrait,T:HasAabb+Copy+'a,N:Copy+'a> TreeInner<A,T,N>{

    pub fn from_dfs_in_order<T2:HasAabb<Num=T::Num>>(axis:A,num_bots:usize,a:&compt::dfs_order::CompleteTree<Node2<T2>>,func: impl Fn(&T2)->T+Copy,n:N)->TreeInner<A,T,N>{

        let num_nodes=a.get_nodes().len();
        let height=a.get_height();
        let bla=a.vistr();

        let mut mem=NodeAllocator::new();
        let root=handle(a.vistr(),&mut mem,func,n);
        let root=(mem.as_ptr() as usize) + (root as usize);
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
                    let node=node.into_node(ll as i32,rr as i32);
                    node.as_ptr() as usize
                },
                None=>{
                    let node=na.create_leaf(nn.mid.iter().map(|a|func(a)),n);
                    node.as_ptr() as usize
                }
            }
        }
    }
}



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


