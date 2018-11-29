use super::*;

use compt::Visitor;
use HasAabb;
use std::marker::PhantomData;
use std::iter::TrustedLen;
use std::ptr::Unique;
use inner_prelude::*;
use tools::*;


#[repr(C)]
struct ReprMut<T>{
    ptr:*mut T,
    size:usize,
}

#[repr(C)]
struct Repr<T>{
    ptr:*const T,
    size:usize,
}



use super::*;
use dinotree_inner::Sorter;



/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct Vistr<'a,N:'a,T:HasAabb+'a>{
    inner:InnerVistr<'a,N,T>
}

impl<'a,N:'a,T:HasAabb+'a> Vistr<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->Vistr<'b,N,T>{
        Vistr{inner:self.inner.create_wrap()}
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
        let (a,b)=self.inner.next();

        let b=match b{
            Some((rest,left,right))=>{
                let left=Vistr{inner:left};
                let right=Vistr{inner:right};

                let fullcomp=if a.dyn.range.len()==0{
                    None
                }else{
                    Some(rest.2)
                };
                Some((fullcomp,left,right))
            },
            None=>{
                None
            }
        };

        (&a.dyn,b)
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}



/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct VistrMut<'a,N:'a,T:HasAabb+'a>{
    inner:InnerVistrMut<'a,N,T>
}

impl<'a,N:'a,T:HasAabb+'a> VistrMut<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        VistrMut{inner:self.inner.create_wrap_mut()}
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
        
        let (a,b)=self.inner.next();

        let b=match b{
            Some((rest,left,right))=>{
                let left=VistrMut{inner:left};
                let right=VistrMut{inner:right};

                let fullcomp=if a.dyn.range.len()==0{
                    None
                }else{
                    Some(rest.2)
                };
                Some((fullcomp,left,right))
            },
            None=>{
                None
            }
        };

        (&mut a.dyn,b)
        
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}





struct InnerVistrMut<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a mut u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a mut (T,N)>
}
impl<'a,N:'a,T:HasAabb+'a> InnerVistrMut<'a,N,T>{
    fn new(ptr:&'a mut u8,max_height:usize)->InnerVistrMut<'a,N,T>{
        InnerVistrMut{ptr,height:max_height,depth:0,_p:PhantomData}
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->InnerVistrMut<'b,N,T>{
        InnerVistrMut{ptr:self.ptr,height:self.height,depth:self.depth,_p:PhantomData}
    }
}
impl<'a,N:'a,T:HasAabb+'a> Visitor for InnerVistrMut<'a,N,T>{
    type Item=&'a mut NodeDynWrap<N,T>;
    type NonLeafItem=(usize,[*mut u8;2],&'a FullComp<T::Num>);
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                let ptr=self.ptr as *mut u8;

                let node=NodeDstDyn::<N,T>::from_ptr_mut(self.ptr,None);

                let nn=(self.height,node.next_nodes,&node.comp);

                let left_pointer:&'a mut u8=unsafe{std::mem::transmute(node.next_nodes[0])};
                let right_pointer:&'a mut u8=unsafe{std::mem::transmute(node.next_nodes[1])};

                let a=InnerVistrMut{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistrMut{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                (&mut node.node,Some((nn,a,b)))
            }else{
                let node=NodeDynWrap::<N,T>::from_ptr_mut(self.ptr,None);
                (node,None)
            }
        }
    }

    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}


struct InnerVistr<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a (T,N)>
}
impl<'a,N:'a,T:HasAabb+'a> InnerVistr<'a,N,T>{
    fn new(ptr:&'a u8,max_height:usize)->InnerVistr<'a,N,T>{
        InnerVistr{ptr,height:max_height,depth:0,_p:PhantomData}
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->InnerVistr<'b,N,T>{
        InnerVistr{ptr:self.ptr,height:self.height,depth:self.depth,_p:PhantomData}
    }
}
impl<'a,N:'a,T:HasAabb+'a> Visitor for InnerVistr<'a,N,T>{
    type Item=&'a NodeDynWrap<N,T>;
    type NonLeafItem=(usize,[*mut u8;2],&'a FullComp<T::Num>);
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                let ptr=self.ptr as *const u8;

                let node=NodeDstDyn::<N,T>::from_ptr(self.ptr,None);

                let nn=(self.height,node.next_nodes,&node.comp);


                let left_pointer:&'a u8=unsafe{std::mem::transmute(node.next_nodes[0])};
                let right_pointer:&'a u8=unsafe{std::mem::transmute(node.next_nodes[1])};

                let a=InnerVistr{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistr{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                (&node.node,Some((nn,a,b)))
            }else{
                let node=NodeDynWrap::<N,T>::from_ptr(self.ptr,None);
                (node,None)
            }
        }
    }

    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
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

///A struct that contains data that only non leaf nodes contain.
#[derive(Copy,Clone)]
pub struct FullComp<N:NumTrait>{
    ///The position of the splitting line for this node.
    pub div:N,
    ///The 1d bounding box for this node. All bots that intersect the splitting line are 
    ///within this bounding box.
    pub cont:axgeom::Range<N> ,

}


struct NodeDstDyn<N,T:HasAabb>{
    //This term can't live in fullcomp, since every if there are no bots in a node, or below,
    //we might want to traverse the lower children to construct the tree properly.
    next_nodes:[*mut u8;2], //offset from parents in terms of bytes

    comp:FullComp<T::Num>,
        
    node:NodeDynWrap<N,T>
}
impl<N,T:HasAabb> NodeDstDyn<N,T>{
    fn from_buffer(buffer:&mut [u8],size:usize)->(&mut NodeDstDyn<N,T>,&mut [u8]){
        assert_eq!((buffer[0] as *mut u8).align_offset(NodeDstDyn::<N,T>::empty_alignment()),0 );
        assert!(buffer.len()<NodeDstDyn::<N,T>::empty_size());
        let node:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut buffer[0],size})};
        //node
        (node,&mut []) //TODO implement
    }
    fn empty_size()->usize{
        let siz={
            let k:&NodeDstDyn<N,T>=unsafe{
                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            std::mem::size_of_val(k)
        };
        siz
    }
    fn empty_alignment()->usize{
        let siz={
            let k:&NodeDstDyn<N,T>=unsafe{
                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            std::mem::align_of_val(k)
        };
        siz
    }

    unsafe fn from_ptr(ptr:&u8,length:Option<usize>)->&NodeDstDyn<N,T>{
        match length{
            None=>{
                let r=Repr{ptr,size:0};
                let k:&NodeDstDyn<N,T>=unsafe{std::mem::transmute(r)};
                let length=k.node.num;
                std::mem::transmute(Repr{ptr,size:length})
            },
            Some(length)=>{
                let r=Repr{ptr,size:length};
                unsafe{std::mem::transmute(r)}
            }
        }
        
    }
    unsafe fn from_ptr_mut(ptr:&mut u8,length:Option<usize>)->&mut NodeDstDyn<N,T>{
        match length{
            None=>{
                let r=ReprMut{ptr,size:0};
                let k:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(r)};
                let length=k.node.num;
                std::mem::transmute(ReprMut{ptr,size:length})
            },
            Some(length)=>{
                let r=ReprMut{ptr,size:length};
                unsafe{std::mem::transmute(r)}
            }
        }
    }
}


struct NodeDynWrap<N,T>{
    num:usize,
    dyn:NodeDyn<N,T>
}

impl<N,T> NodeDynWrap<N,T>{
    fn from_buffer(buffer:&mut [u8],size:usize)->(&mut NodeDynWrap<N,T>,&mut [u8]){
        assert!(NodeDynWrap::<N,T>::empty_size()<=buffer.len(),"{:?}",(NodeDynWrap::<N,T>::empty_size(),buffer.len()));
        
        assert_eq!((buffer.as_mut_ptr()).align_offset(NodeDynWrap::<N,T>::empty_alignment()),0 );
        let node:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut buffer[0],size})};
        //node
        (node,&mut []) //TODO implement
    }

    fn empty_alignment()->usize{
        let siz={
            let k:&NodeDynWrap<N,T>=unsafe{
                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            std::mem::align_of_val(k)
        };
        siz
    }
    fn empty_size()->usize{
        let siz={
            let k:&NodeDynWrap<N,T>=unsafe{
                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            std::mem::size_of_val(k)
        };
        siz
    }

    unsafe fn from_ptr(ptr:&u8,length:Option<usize>)->&NodeDynWrap<N,T>{
        match length{
            None=>{
                let r=Repr{ptr,size:0};
                let k:&NodeDynWrap<N,T>=unsafe{std::mem::transmute(r)};
                let length=k.num;
                std::mem::transmute(Repr{ptr,size:length})
            },
            Some(length)=>{
                let r=Repr{ptr,size:length};
                unsafe{std::mem::transmute(r)}
            }
        }

    }
    unsafe fn from_ptr_mut(ptr:&mut u8,length:Option<usize>)->&mut NodeDynWrap<N,T>{
        match length{
            None=>{
                let r=ReprMut{ptr,size:0};
                let k:&NodeDynWrap<N,T>=unsafe{std::mem::transmute(r)};
                let length=k.num;
                std::mem::transmute(ReprMut{ptr,size:length})
            },
            Some(length)=>{
                let r=ReprMut{ptr,size:length};
                unsafe{std::mem::transmute(r)}
            }
        }
    }
}





trait LeftOrRight{
    fn bots_is_right_side_of_buffer(&self)->bool;
}
struct LeftOf;
impl LeftOrRight for LeftOf{
    fn bots_is_right_side_of_buffer(&self)->bool{
        return false;
    }   
}
struct RightOf;
impl LeftOrRight for RightOf{
    fn bots_is_right_side_of_buffer(&self)->bool{
        return true;
    }   
}



fn construct_leaf<T:HasAabb>(sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T]){ 
    sorter.sort(div_axis.next(),bots);
}

fn construct_non_leaf<T:HasAabb>(sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T])->Option<(FullComp<T::Num>,&mut [T],&mut [T],&mut [T])>{
    let med=if bots.len() == 0{
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


    for a in bots.iter(){ //TODO remove
        let a=a.get().get_range(div_axis);
        assert!(a.left<=a.right);
    }
    
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned=oned::bin_left_middle_right(div_axis,&med,bots);

    assert!(binned.middle.len()!=0);
    
    //We already know that the middile is non zero in length.
    let container_box=dinotree_inner::create_cont(div_axis,binned.middle).unwrap();
    
    //oned::sweeper_update(div_axis.next(),binned_middle);
    sorter.sort(div_axis.next(),binned.middle);
    let full=FullComp{div:med,cont:container_box};
    Some((full,binned.left,binned.middle,binned.right))
}

/*
//returns the number of bytes that need to be reserved for the number of bots and nodes that are left.
fn compute_space<N,T:HasAabb>(num_bots:usize,height:usize,max_height:usize)->usize{
    //  |-----------------------------------------------|
    //                     |----left--|--mid--|----right|

    let number_of_levels_left=max_height-height;
    let num_nodes_under=2usize.rotate_left(number_of_levels_left as u32);

    let num_non_leafs=num_nodes_under/2;
    let num_leafs=num_nodes_under-num_non_leafs;

    num_bots*std::mem::size_of::<T>()+NodeDstDyn::<N,T>::empty_size()*num_non_leafs+NodeDynWrap::<N,T>::empty_size()*num_leafs
}
*/


unsafe impl<T:HasAabb,N> Send for TreeInner<T,N>{}

unsafe impl<T:HasAabb,N> Sync for TreeInner<T,N>{}


pub struct TreeInner<T:HasAabb,N>{
    mem:chunk::MemChunk,
    target:*mut u8,
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    _p:PhantomData<(T,N)>
}



fn compute_size<N,T:HasAabb>(num_bots:usize,height:usize,)->(usize,usize){
    use std::mem::*;
    let dst:&NodeDstDyn<N,T>=unsafe{
        let k:*const u8=std::mem::transmute(0x10 as usize);
        std::mem::transmute(Repr{ptr:k,size:0})
    };

    let wrap:&NodeDynWrap<N,T>=unsafe{
        let k:*const u8=std::mem::transmute(0x10 as usize);
        std::mem::transmute(Repr{ptr:k,size:0})
    };

    //TODO do this!
    //assert_eq!(align_of::<T>(),align_of_val(dst));
    //assert_eq!(align_of::<T>(),align_of_val(wrap));

    println!("size of T={:?} size of dstdyn={:?} size of dynwrap={:?}",size_of::<T>(),NodeDstDyn::<N,T>::empty_size(),NodeDynWrap::<N,T>::empty_size());
    println!("num bots={:?} height={:?}",num_bots,height);

    //TODO also assert here?
        
    let num_nodes=1usize.rotate_left(height as u32)-1; //TODO verify

    let num_nodes=num_nodes*2;

    let num_non_leafs=num_nodes/2;
    let num_leafs=num_nodes-num_non_leafs;


    let num_bytes=
        num_bots*(size_of::<T>())+
        num_non_leafs * (NodeDstDyn::<N,T>::empty_size())+
        num_leafs * (NodeDynWrap::<N,T>::empty_size());


    (num_nodes,num_bytes)
}


impl<T:HasAabb+Copy,N:Copy> TreeInner<T,N>{
    pub fn new(sorter:impl Sorter,axis:impl AxisTrait,bbots:&[T],n:N,height:usize)->TreeInner<T,N>
    {
        //TODO do this!
        //assert_eq!(align_of::<T>(),align_of::<NodeDstDyn2<N,T::Num>>());
        //assert_eq!(align_of::<T>(),align_of::<NodeDynWrap<N>>());

        //assert_eq!(size_of::<T>(),align_of::<NodeDstDyn2<N,T::Num>>());
        //assert_eq!(align_of::<T>(),align_of::<NodeDynWrap<N>>());
                

        let num_bots=bbots.len();

        let (num_nodes,num_bytes)=compute_size::<N,T>(bbots.len(),height);
        //TODO remove zeroing out.
        // let mut mem:Vec<u8>=std::iter::repeat(0).take(num_bytes).collect();
        let mut mem=chunk::MemChunk::new(num_bytes,std::mem::align_of::<T>());

        let target=unsafe{
            let (bots,buffer)={
                let (_,bots,c)=mem.get_mut().align_to_mut::<T>();
                let (bots,rest)=bots.split_at_mut(num_bots);
                let buffer=slice_join_bytes_mut(rest,c);
                bots[..].copy_from_slice(bbots);
                (bots,buffer)
            };
            handle_node(sorter,axis,LeftOf,bots,buffer,n,0,height)
            
        };

        TreeInner{num_bots,num_nodes,mem,target,height,_p:PhantomData}

    }
}


struct NodeAllocator<'a,N,T>{
    mem:chunk::MemChunk,
    counter:usize,
    _p:PhantomData<&'a mut (N,T)>
}
impl<'a,N,T:HasAabb> NodeAllocator<'a,N,T>{
    fn new(num_bots:usize,height:usize)->NodeAllocator<'a,N,T>{
        let (num_nodes,num_bytes)=compute_size::<N,T>(num_bots,height);
        let mut mem=chunk::MemChunk::new(num_bytes,std::mem::align_of::<T>());

        NodeAllocator{
            mem,counter:0,_p:PhantomData
        }
    }
    fn into_inner(self)->chunk::MemChunk{
        self.mem
    }
    fn create_non_leaf(&mut self,num:usize)->&'a mut NodeDstDyn<N,T>{
        unsafe{
            let ptr=self.mem.as_mut_ptr().offset(self.counter as isize);
            let node:&'a mut NodeDstDyn<N,T>=std::mem::transmute(ReprMut{ptr,size:num});
            self.counter+=std::mem::size_of_val(node);
            assert!(self.counter<self.mem.capacity());
            node
        }
    }
    fn create_leaf(&mut self,num:usize)->&'a mut NodeDynWrap<N,T>{
        unsafe{
            let ptr=self.mem.as_mut_ptr().offset(self.counter as isize);
            let node:&'a mut NodeDynWrap<N,T>=std::mem::transmute(ReprMut{ptr,size:num});
            self.counter+=std::mem::size_of_val(node);
            assert!(self.counter<self.mem.capacity());
            node
        }
    }
}


impl<T:HasAabb,N> TreeInner<T,N>{
    pub fn into_other<T2:HasAabb<Num=T::Num>,N2>(&self,mut func1:impl FnMut(&T)->T2,mut func2:impl FnMut(&N)->N2)->TreeInner<T2,N2>{
        //TODO check assertions here!
        use std::mem::*;

        
        let a=InnerVistr::new(unsafe{std::mem::transmute(self.target as *const u8)},self.height);
        
        let mut allocator=NodeAllocator::<N2,T2>::new(self.num_bots,self.height);

        let start=handle(&mut allocator,a,&mut func1,&mut func2);


        return TreeInner{mem:allocator.into_inner(),target:start,height:self.height,num_nodes:self.num_nodes,num_bots:self.num_bots,_p:PhantomData};
        


        fn handle<T:HasAabb,T2:HasAabb<Num=T::Num>,N,N2>(allocator:&mut NodeAllocator<N2,T2>,a:InnerVistr<N,T>,func1:&mut impl FnMut(&T)->T2,func2:&mut impl FnMut(&N)->N2)->*mut u8{

            let (original,rest)=a.next();


            match rest{
                Some(((depth,next_nodes,fullcomp),left,right))=>{

                    let a=handle(allocator,left,func1,func2);
                    
                    let node2=allocator.create_non_leaf(original.num);


                    let b=handle(allocator,right,func1,func2);

                    node2.comp=*fullcomp;
                    node2.next_nodes=[a,b];
                    {
                        let no=&mut node2.node;
                        no.num=original.num;
                        no.dyn.misc=func2(&original.dyn.misc);
                        for (a,b) in original.dyn.range.iter().zip(no.dyn.range.iter_mut()){
                            *b=func1(a);
                        }
                    }

                
                    //set node
                    //this_counter
                    
                    let ReprMut{ptr,size}=unsafe{std::mem::transmute(node2)};
                    ptr

                },
                None=>{
                    
                    let node2=allocator.create_leaf(original.num);

                    node2.num=original.num;
                    node2.dyn.misc=func2(&original.dyn.misc);
                    for (a,b) in original.dyn.range.iter().zip(node2.dyn.range.iter_mut()){
                        *b=func1(a);
                    }

                    let ReprMut{ptr,size}=unsafe{std::mem::transmute(node2)};
                    ptr
                    
                }
            }
        }
    }
    
    pub fn num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn vistr(&self)->Vistr<N,T>{
        unsafe{
            let buffer=self.mem.get();
            
            let bot_size=std::mem::size_of::<(N,T)>();
            //let ptr=&buffer[self.target as usize*bot_size];
            let ptr=std::mem::transmute(self.target);
            let inner=InnerVistr::new(ptr,self.height);
            Vistr{inner}
        }
    }
    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        unsafe{
            let buffer=self.mem.get_mut();

            let bot_size=std::mem::size_of::<(N,T)>();
            let ptr=std::mem::transmute(self.target);//&mut buffer[self.target as usize*bot_size];
            let inner=InnerVistrMut::new(ptr,self.height);
            VistrMut{inner}
        }
    }
   
}


fn handle_node<T:HasAabb+Copy,N:Copy,S:Sorter,A:AxisTrait,L:LeftOrRight>(sorter:S,axis:A,st:L,bots:&mut [T],buffer:&mut [u8],n:N,depth:usize,height:usize)->*mut u8
{
    if st.bots_is_right_side_of_buffer(){
        assert!(are_adjacent(buffer,bots));
    }else{
        assert!(are_adjacent(bots,buffer));
    }
    
    let bot_size=std::mem::size_of::<T>();

    if depth<height-1{
        
        let (fullcomp,left,mid,right)={
                        
            let (fullcomp,left,mid,right)=match construct_non_leaf(sorter,axis,bots){
                Some(pass)=>{
                    pass
                },
                None=>{
                    let d=unsafe{
                        let mut d=std::mem::uninitialized();
                        std::ptr::write_bytes(&mut d,0,std::mem::size_of::<T::Num>());
                        d                        
                    };

                    let (empty1,empty2,empty3)={
                        let target=if st.bots_is_right_side_of_buffer(){
                            let len=buffer.len();
                            buffer[len..].as_mut_ptr()
                        }else{
                            buffer.as_mut_ptr()
                        };
                        let target=target as *mut T;
                        unsafe{
                            let empty1:&mut [T]=std::slice::from_raw_parts_mut(target,0);
                            let empty2:&mut [T]=std::slice::from_raw_parts_mut(target,0);
                            let empty3:&mut [T]=std::slice::from_raw_parts_mut(target,0);
                            (empty1,empty2,empty3)
                        }
                    };

                    (
                        FullComp{div:d,cont:Range{left:d,right:d}},
                        empty1,
                        empty2,
                        empty3
                    )
                }
            };

            (fullcomp,left,mid,right)
        };



        
        let (left_buffer,left,_a,node,right,right_buffer)=if st.bots_is_right_side_of_buffer(){
            move_bots_non_leaf(depth,height,false,left,mid,right,buffer)
        }else{
            move_bots_non_leaf(depth,height,true,left,mid,right,buffer)
        };

        //println!("len={:?}",_a.len());
         
        //Construct this node.
        let left_node =handle_node(sorter,axis.next(),RightOf,left,left_buffer,n,depth+1,height);
        let right_node=handle_node(sorter,axis.next(),LeftOf,right,right_buffer,n,depth+1,height);

        
        node.comp=fullcomp;
        node.next_nodes=[left_node,right_node];
        node.node.dyn.misc=n;
        node.node.num=node.node.dyn.range.len();//mid.len();
        
        
        let ReprMut{ptr,size}=unsafe{std::mem::transmute(node)};
        ptr
    }
    else
    {
        construct_leaf(sorter,axis,bots);

        let (_left_buffer,node,_right_buffer)=if st.bots_is_right_side_of_buffer(){
            move_bots_leaf(false,bots,buffer)
        }else{
            move_bots_leaf(true,bots,buffer)
        };
        //println!("leftover space={:?}",(_left_buffer.len(),_right_buffer.len()));
        node.dyn.misc=n;
        node.num=node.dyn.range.len();

        let ReprMut{ptr,size}=unsafe{std::mem::transmute(node)};
        ptr
    }
}



#[test]
fn move_bots_leaf_test(){

    let mut bots:Vec<BBox<u8,()>>=(0..40).map(|a|unsafe{BBox::new(axgeom::Rect::new(0xDEADu8,0xBEAF,0xCAFE,0xBABE),())}).collect();
    {
        let (bots,rest)=bots.split_at_mut(10);

        let rest:&mut [u8]=unsafe{
            let r=ReprMut{ptr:rest.as_mut_ptr() as *mut u8,size:rest.len()*std::mem::size_of::<BBox<u8,()>>()};
            std::mem::transmute(r)
        };

        
        let (a,b,c)=move_bots_leaf::<(),_>(true,bots,rest);
        b.num=0;

        //println!("sizes={:?}",(a.len(),c.len()));
        for a in a.iter_mut(){
            *a=0;
        }
        for a in c.iter_mut(){
            *a=0;
        }
    }

    for a in bots{
        let ((a,b),(c,d))=a.get().get();
        println!("{:x}{:x}{:x}{:x}",a,b,c,d);
    }
    //println!("bots={:?}",bots);
    panic!();
}

fn move_bots_leaf<'a,N,T:HasAabb+Copy>(move_right:bool,bots:&'a mut [T],rest:&'a mut [u8])->(&'a mut [u8],&'a mut NodeDynWrap<N,T>,&'a mut [u8]){
    let bots_copy:Vec<T>=bots.iter().map(|a|*a).collect();
    
    unsafe{
        use std::mem::*;
        let (total_size_of_mid,align_of_node)={
            let val:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x10 as *mut u8,size:bots.len()})};
            //assert_eq!(align_of::<T>(),align_of_val(val));
            (size_of_val(val),align_of_val(val))
        };

        let(start,end)=if move_right{
            assert!(are_adjacent(bots,rest));
            let start=bots.as_ptr();
            let len=rest.len();
            let end=rest[len..].as_ptr();
            (start as usize,end as usize)
        }else{
            assert!(are_adjacent(rest,bots));        
            let start=rest.as_ptr();
            let len=bots.len();
            let end=bots[len..].as_ptr();
            (start as usize,end as usize)
        };

        let target=start as *mut u8;
        let offset=target.align_offset(align_of_node);
        let target=target.offset(offset as isize);


        let val:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:target,size:bots.len()})};
        std::ptr::copy(bots.as_ptr(),val.dyn.range.as_mut_ptr(),bots.len());
        //val.dyn.range.copy_from_slice(&bots);


        let left_buffer_len=(target as usize) - start;

        let right_buffer_start=target.offset(size_of_val(val) as isize);
        let right_buffer_len=end-(right_buffer_start as usize);

        let left=std::slice::from_raw_parts_mut(start as *mut u8,left_buffer_len);
        let right=std::slice::from_raw_parts_mut(right_buffer_start,right_buffer_len);

        assert_eq!(left.as_ptr() as usize,start);
        assert_eq!((right[right_buffer_len..].as_ptr() as usize),end);
        assert_eq!(left[left_buffer_len..].as_ptr(),target);
        assert_eq!(right.as_ptr(),target.offset(total_size_of_mid as isize));

        for (a,b) in bots_copy.iter().zip(val.dyn.range.iter()){
            assert!(a.get().equals(b.get()));
        }
        
        (left,val,right)
    }
}


#[test]
fn move_bots_non_leaf_test(){

    let mut bots:Vec<BBox<u8,()>>=(0..40).map(|a|unsafe{BBox::new(axgeom::Rect::new(0xDEADu8,0xBEAF,0xCAFE,0xBABE),())}).collect();
    {
        let (bots,rest)=bots.split_at_mut(10);

        let rest:&mut [u8]=unsafe{
            let r=ReprMut{ptr:rest.as_mut_ptr() as *mut u8,size:rest.len()*std::mem::size_of::<BBox<u8,()>>()};
            std::mem::transmute(r)
        };

        
        let (buffer_left,left,unused_left,node,unused_right,right,buffer_right)=move_bots_non_leaf::<(),_>(true,bots,rest);
        b.num=0;

        //println!("sizes={:?}",(a.len(),c.len()));
        for a in buffer_left.iter_mut(){
            *a=0;
        }
        for a in buffer_right.iter_mut(){
            *a=0;
        }
    }

    for a in bots{
        let ((a,b),(c,d))=a.get().get();
        println!("{:x}{:x}{:x}{:x}",a,b,c,d);
    }
    //println!("bots={:?}",bots);
    panic!();
}

fn calculate_space_needed<N,T:HasAabb>(depth:usize,height:usize,num_bots:usize)->usize{
    use std::mem::*;
    let number_of_levels_left=height-depth;

    let num_nodes_left=2usize.rotate_left(number_of_levels_left as u32)-1;

    //let num_nodes_left=(num_nodes_left as f64*1.22 ) as usize;
    let num_nodes_left=num_nodes_left/2;
    //The number of nodes on the left side only.
    //let num_nodes_left=num_nodes_left/2;

    let num_non_leafs=num_nodes_left/2;
    let num_leafs=num_nodes_left-num_non_leafs;

    let k=num_bots*std::mem::size_of::<T>()+
    (NodeDstDyn::<N,T>::empty_size())*num_non_leafs+
    (NodeDynWrap::<N,T>::empty_size())*num_leafs;

    k

}

fn move_bots_non_leaf<'a,N,T:HasAabb+Copy>(depth:usize,height:usize,move_right:bool,left:&'a mut [T],mid:&'a mut [T],right:&'a mut [T],rest:&'a mut [u8])->(&'a mut [u8],&'a mut [T],&'a mut [u8],&'a mut NodeDstDyn<N,T>,&'a mut [T],&'a mut [u8]){
    /*
    let target={
        //let ratio=left/right;
        //let total_size=all memory we have;
        //
        //find x such that
        /*
        x/y=ratio;
        x+y=total_size;   y=total_size-x;

        x/(total_size-x)=ratio;
        x=ratio*(total_size-x);
        x=ratio*total_size-ratio*x;
        ratio*total_size=x+ratio*x;
        ratio*total_size=x*(1+ratio);
        x=ratio*total_size/(1+ratio);
        */



    }
    */
    let space_needed_for_left_bots=calculate_space_needed::<N,T>(depth,height,left.len());




    let left_copy:Vec<T>=left.iter().map(|a|*a).collect();
    let right_copy:Vec<T>=right.iter().map(|a|*a).collect();

    unsafe{
        use std::mem::*;
        let bot_size=size_of::<T>();
        let (total_size_of_mid,align_of_node)={
            let val:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:0x128 as *mut u8,size:mid.len()})};
      
            //println!("size={:?}  node={:?}",std::mem::size_of::<T>(),NodeDstDyn::<N,T>::empty_size());
            assert_eq!(val.node.dyn.range.len(),mid.len());
            //assert_eq!(align_of::<T>(),align_of_val(val));
            (size_of_val(val),align_of_val(val))
        };
        assert!(are_adjacent(left,mid));
        assert!(are_adjacent(mid,right));
        

        let(start,end)=if move_right{
            assert!(are_adjacent(right,rest));
            let start=left.as_ptr();
            let len=rest.len();
            let end=rest[len..].as_ptr();
            (start as usize,end as usize)
        }else{
            assert!(are_adjacent(rest,left));        
            let start=rest.as_ptr();
            let len=right.len();
            let end=right[len..].as_ptr();
            (start as usize,end as usize)
        };

        //let diff=(end- start)/2;  
        //we have a target now. Now find the closest pointer that aligns to it.
        //let target=start+diff-(total_size_of_mid/2);
        let target=start+space_needed_for_left_bots;


        let target=target as *mut u8;
        let target:*mut u8=target.offset(target.align_offset(align_of_node) as isize);


        //Now we have an aligned target for the mid.


        let val:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:target,size:mid.len()})};
        


        //We have a left start.
        let left_start:*mut T={
            let left_start=target.offset(-((left.len()*bot_size) as isize));
            let left_start=left_start.offset(-(align_of::<T>() as isize));
            //Now align it to the left.
            let bot_off=left_start.align_offset(align_of::<T>());
            let left_start=left_start.offset(bot_off as isize);
            left_start as *mut T
        };



        
        let right_start:*mut T={
            val.node.dyn.range.as_mut_ptr().offset(mid.len() as isize)
            /*
            let right_start=target.offset(size_of_val(val) as isize);
            let offset=right_start.align_offset(align_of::<T>());
            assert_eq!(offset,0);
            right_start.offset(offset as isize) as *mut T
            */
        };


        //aparently this isnt true????
        //So the dst overlapps a bit with the slice we return,
        //but this is oaky since it must be just padding??
        //assert_eq!(right_start as *mut u8,target.offset(size_of_val(val) as isize));

        
        //At this point we havent written anything. Before we do, lets make sure we can fit everything.
        assert!(left_start as usize>=start);
        let right_end=(right_start.offset(right.len() as isize) as usize);
        assert!(right_end <=end);


        //Now move the bots
        if move_right{
            
            std::ptr::copy(mid.as_mut_ptr(),val.node.dyn.range.as_mut_ptr(),mid.len()+right.len());

            std::ptr::copy(left.as_mut_ptr(),left_start,left.len());
        }else{

            std::ptr::copy(left.as_mut_ptr(),left_start,left.len());
            
            std::ptr::copy(mid.as_mut_ptr(),val.node.dyn.range.as_mut_ptr() ,mid.len()+right.len());
        }

        let left_new:&mut [T]=std::slice::from_raw_parts_mut(left_start as *mut T,left.len());
        let right_new:&mut [T]=std::slice::from_raw_parts_mut(right_start as *mut T,right.len());

        drop(left);
        drop(right);


        let diff=(target as usize) - (left_start.offset(left_new.len() as isize) as usize);
        let unused_left:&mut [u8]=std::slice::from_raw_parts_mut(left_start.offset(left_new.len() as isize) as *mut u8,diff);

        let unused_left_len=unused_left.len();
        assert_eq!(unused_left[unused_left_len..].as_ptr(),target);

        /*
        let target_end=target.offset(total_size_of_mid as isize);
        let diff=(right_start as usize) - (target_end as usize);
        let unused_right:&mut [u8]=std::slice::from_raw_parts_mut(target_end as *mut u8,diff);
        assert_eq!(target.offset(total_size_of_mid as isize) as *const u8,unused_right.as_ptr());
        assert_eq!(unused_right.len(),0);
        */

        let left_buffer_size=left_start as usize - start;
        let left_buffer:&mut [u8]=std::slice::from_raw_parts_mut(start as *mut u8,left_buffer_size);
        let right_buffer_size=end-right_end;
        let right_buffer:&mut [u8]=std::slice::from_raw_parts_mut(right_end as *mut u8,right_buffer_size);
        


        assert!(are_adjacent(left_new,unused_left));
        //assert!(are_adjacent(unused_right,right_new));

        assert!(are_adjacent(left_buffer,left_new));
        assert!(are_adjacent(right_new,right_buffer));

        for (a,b) in left_copy.iter().zip(left_new.iter()){
            assert!(a.get().equals(b.get()));
        }
        for (a,b) in right_copy.iter().zip(right_new.iter()){
            assert!(a.get().equals(b.get()),"{:?}",move_right);
        }


        //println!("sizes={:?}",( bytes_join_slice_mut(left_buffer,left_new).len(),slice_join_bytes_mut(right_new,right_buffer).len()  ));
        (left_buffer,left_new,unused_left,val,right_new,right_buffer)
        
    }
}





mod chunk{
    use tree_alloc;
    pub struct MemChunk{
        vec:Vec<u8>,
        offset:isize,
        num_bytes:usize
    }
    impl MemChunk{
        pub unsafe fn get_ptr_mut(&mut self)->*mut u8{
            &mut self.vec[0] as *mut u8
        }
        pub fn get(&self)->&[u8]{
            let offset=self.offset;
            let num_bytes=self.num_bytes;
            unsafe{
                let a=self.vec.as_ptr().offset(offset);
                std::mem::transmute(tree_alloc::Repr{ptr:a,size:self.num_bytes})
            }
        }
        pub fn as_mut_ptr(&mut self)->*mut u8{
            self.vec.as_mut_ptr()
        }
        pub fn capacity(&self)->usize{
            self.num_bytes
        }
        pub fn get_mut(&mut self)->&mut [u8]{
            let offset=self.offset;
            let num_bytes=self.num_bytes;
            unsafe{
                let a=self.vec.as_mut_ptr().offset(offset);
                std::mem::transmute(tree_alloc::ReprMut{ptr:a,size:self.num_bytes})
            }
        }
        pub fn new(num_bytes:usize,alignment:usize)->MemChunk{


            let (offset,vec)={

                let mut vec=Vec::with_capacity(alignment+num_bytes);

            
                let mut counter=vec.as_ptr() as *mut u8;

                let offset=counter.align_offset(alignment);
                if offset==usize::max_value(){
                    panic!("Error finding alignment!");
                }


                (offset as isize,vec)
            };

            if num_bytes %alignment!=0{
                panic!("fail!");
            }
            MemChunk{vec,offset,num_bytes}
        }

    }
}
