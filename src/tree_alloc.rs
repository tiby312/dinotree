use super::*;

use compt::Visitor;
use HasAabb;
use std::marker::PhantomData;
use std::iter::TrustedLen;
use std::ptr::Unique;
use inner_prelude::*;


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
                //let node:&'a mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.ptr,size:0})};
                //let length=node.node.num;
                //let node:&'a mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.ptr,size:length})};
                let node=NodeDstDyn::<N,T>::from_ptr_mut(self.ptr,None);


                let nn=(self.height,node.next_nodes,&node.comp);



                let bot_size=std::mem::size_of::<T>() as isize;
                let left_pointer=(ptr as *mut u8).offset(-(node.next_nodes[0] as isize)*bot_size);

                let node_size=std::mem::size_of_val(node) as isize;
                //let kkk=number_of_bots_to_cover_non_leaf_node_data::<N,T>() as isize+length as isize+node.next_nodes[1];
                let right_pointer=(ptr as *mut u8).offset(node_size+(node.next_nodes[1] as isize)*bot_size);

                let left_pointer:&'a mut u8=unsafe{std::mem::transmute(left_pointer)};
                let right_pointer:&'a mut u8=unsafe{std::mem::transmute(right_pointer)};

                let a=InnerVistrMut{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistrMut{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                //let stuff=(nn,VistrMut{ptr:&mut node.next_nodes.0,depth:self.depth+1,height},
                //VistrMut{ptr:&mut node.next_nodes.1,depth:self.depth+1,height});
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
                //let node:&'a mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.ptr,size:0})};
                //let length=node.node.num;
                //let node:&'a mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.ptr,size:length})};
                let node=NodeDstDyn::<N,T>::from_ptr(self.ptr,None);


                let nn=(self.height,node.next_nodes,&node.comp);



                let bot_size=std::mem::size_of::<T>() as isize;
                let left_pointer=(ptr as *const u8).offset(- (node.next_nodes[0] as isize)*bot_size);

                let node_size=std::mem::size_of_val(node) as isize;
                //let kkk=number_of_bots_to_cover_non_leaf_node_data::<N,T>() as isize+length as isize+node.next_nodes[1];
                let right_pointer=(ptr as *const u8).offset(node_size+(node.next_nodes[1] as isize)*bot_size);

                let left_pointer:&'a  u8=unsafe{std::mem::transmute(left_pointer)};
                let right_pointer:&'a u8=unsafe{std::mem::transmute(right_pointer)};

                let a=InnerVistr{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistr{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                //let stuff=(nn,VistrMut{ptr:&mut node.next_nodes.0,depth:self.depth+1,height},
                //VistrMut{ptr:&mut node.next_nodes.1,depth:self.depth+1,height});
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
    fn from_buffer(buffer:&mut [u8],size:usize)->&mut NodeDstDyn<N,T>{
        assert_eq!((buffer[0] as *mut u8).align_offset(NodeDstDyn::<N,T>::empty_alignment()),0 );
        assert!(buffer.len()<NodeDstDyn::<N,T>::empty_size());
        let node:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut buffer[0],size})};
        node
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
    fn from_buffer(buffer:&mut [u8],size:usize)->&mut NodeDynWrap<N,T>{
        assert_eq!((buffer[0] as *mut u8).align_offset(NodeDynWrap::<N,T>::empty_alignment()),0 );
        assert!(buffer.len()<NodeDynWrap::<N,T>::empty_size());
        let node:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut buffer[0],size})};
        node
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




fn construct<T:HasAabb>(sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T])->Option<(FullComp<T::Num>,&mut [T],&mut [T],&mut [T])>{
    println!("construct");
    let med=if bots.len() == 0{
        return None;
    }
    else
    {
        println!("a");
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
    println!("a");
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned=oned::bin_middle_left_right(div_axis,&med,bots);
    println!("a");
    debug_assert!(binned.middle.len()!=0);
    
    println!("a");
    //We already know that the middile is non zero in length.
    let container_box=dinotree_inner::create_cont(div_axis,binned.middle).unwrap();
    
    //oned::sweeper_update(div_axis.next(),binned_middle);
    sorter.sort(div_axis.next(),binned.middle);
    println!("a");
    let full=FullComp{div:med,cont:container_box};
    println!("construct fin");
    Some((full,binned.left,binned.middle,binned.right))
}

/*
pub fn number_of_bots_to_cover_non_leaf_node_data<N,T:HasAabb>()->usize{
    let a=std::mem::size_of::<NodeDstDyn<N,T>>();
    let b=std::mem::size_of::<T>();
    let mut res=a/b;
    if a%b>0{
        res+=1
    }
    res
}
pub fn number_of_bots_to_cover_leaf_node_data<N,T:HasAabb>()->usize{
    let a=std::mem::size_of::<NodeDynWrap<N>>();
    let b=std::mem::size_of::<T>();
    let mut res=a/b;
    if a%b>0{
        res+=1
    }
    res
}
*/

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
    assert_eq!(align_of::<T>(),align_of_val(dst));
    assert_eq!(align_of::<T>(),align_of_val(wrap));

    println!("size of T={:?} size of dstdyn={:?} size of dynwrap={:?}",size_of::<T>(),NodeDstDyn::<N,T>::empty_size(),NodeDynWrap::<N,T>::empty_size());
    println!("num bots={:?} height={:?}",num_bots,height);

    //TODO also assert here?
        
    let num_nodes=1usize.rotate_left(height as u32)-1; //TODO verify

    let num_non_leafs=num_nodes/2;
    let num_leafs=num_nodes-num_non_leafs;


    let num_bots_total=
        num_bots+
        num_non_leafs * NodeDstDyn::<N,T>::empty_size()+
        num_leafs * NodeDynWrap::<N,T>::empty_size();

    let num_bytes=num_bots_total*size_of::<T>();
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
        println!("num_bytes={:?}",num_bytes);
        //TODO remove zeroing out.
        // let mut mem:Vec<u8>=std::iter::repeat(0).take(num_bytes).collect();
        let mut mem=chunk::MemChunk::new(num_bytes,std::mem::align_of::<T>());

        let target=unsafe{
            let (bots,buffer)={
                let (_,bots,c)=mem.get_mut().align_to_mut::<T>();
                let (bots,rest)=bots.split_at_mut(num_bots);
                let buffer=slice_join_mut2(rest,c);
                bots[..].copy_from_slice(bbots);
                (bots,buffer)
            };
            handle_node(sorter,axis,LeftOf,bots,buffer,n,0,height)
        };

        TreeInner{num_bots,num_nodes,mem,target,height,_p:PhantomData}

    }
}



impl<T:HasAabb,N> TreeInner<T,N>{
    pub fn into_other<T2:HasAabb<Num=T::Num>,N2>(&self,mut func1:impl FnMut(&T)->T2,mut func2:impl FnMut(&N)->N2)->TreeInner<T2,N2>{
        //TODO check assertions here!
        use std::mem::*;
        //let original_bot_size=std::mem::size_of::<T>();
        //let target_bot_size=std::mem::size_of::<T2>();

        let orig_buffer=&self.mem.get();

        //TODO figure out

        let (num_nodes,num_bytes)=compute_size::<N2,T2>(self.num_bots,self.height);
        //let mut target=Vec::with_capacity(num_bytes);
        //let mut target:Vec<u8>=std::iter::repeat(0).take(num_bytes).collect();
        let mut target=chunk::MemChunk::new(num_bytes,std::mem::align_of::<T2>());

        let a=InnerVistr::new(&orig_buffer[0],self.height);
        
        let mut target_counter=0;
        
        let start=handle(target.get_mut(),&mut target_counter,a,&mut func1,&mut func2);

        return TreeInner{mem:target,target:start,height:self.height,num_nodes:self.num_nodes,num_bots:self.num_bots,_p:PhantomData};
        


        fn handle<T:HasAabb,T2:HasAabb<Num=T::Num>,N,N2>(target_buffer:&mut [u8],target_counter:&mut usize,a:InnerVistr<N,T>,func1:&mut impl FnMut(&T)->T2,func2:&mut impl FnMut(&N)->N2)->*mut u8{

            let (original,rest)=a.next();


            match rest{
                Some(((depth,next_nodes,fullcomp),left,right))=>{

                    let a=handle(target_buffer,target_counter,left,func1,func2);
                    
                    let this_counter={
                        let this_counter=*target_counter;
                        let node2=unsafe{NodeDstDyn::<N2,T2>::from_ptr_mut(&mut target_buffer[this_counter],Some(original.num))};
                        *target_counter+=size_of_val(node2);
                        this_counter
                    };

                    let b=handle(target_buffer,target_counter,right,func1,func2);

  
                
                    let node2=unsafe{NodeDstDyn::<N2,T2>::from_ptr_mut(&mut target_buffer[this_counter],Some(original.num))};
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
                    let this_counter=*target_counter;
                    
                    let node2=unsafe{NodeDynWrap::<N2,T2>::from_ptr_mut(&mut target_buffer[this_counter],Some(original.num))};
                    *target_counter+=size_of_val(node2);
                    

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
        /*
        {
            let target_buffer=&mut target;
            
            let mut original_counter=0;
            let mut target_counter=0;
            let leaf_counter=true;

            
            for (orig_node,extra) in InnerVistr::new(&orig_buffer[0],self.height).dfs_inorder_iter(){
                let ptr=&mut target_buffer[target_counter];
                let node2=match extra{
                    Some((height,next_nodes,fullcomp))=>{
                        let node2=unsafe{NodeDstDyn::<N2,T2>::from_ptr_mut(ptr,Some(orig_node.num))};
                        target_counter+=size_of_val(node2);

                        node2.next_nodes[0]=convert_target(0,height,max_height);
                        node2.next_nodes[1]=convert_target(0,height,max_height);
                        node2.comp=*fullcomp;
                        &mut node2.node
                        
                    },
                    None=>{
                        let node2=unsafe{NodeDynWrap::<N2,T2>::from_ptr_mut(ptr,Some(orig_node.num))};
                        target_counter+=size_of_val(node2);
                        node2
                    }
                };

                node2.num=orig_node.num;
                node2.dyn.misc=func2(&orig_node.dyn.misc);
                for (a,b) in orig_node.dyn.range.iter().zip(node2.dyn.range.iter_mut()){
                    *b=func1(a);
                }
            }
        }

        return TreeInner{mem:target,target:convert_target(self.target),height:self.height,num_nodes:self.num_nodes,num_bots:self.num_bots,_p:PhantomData};
        
        fn convert_target(original_offset:isize)->isize{
            0
        }
        */
        /*
        for n in 0..self.num_nodes{
            let (node,node2,orig_size,target_size)=if leaf_counter{
                let node=NodeDynWrap::<N,T>::from_ptr(&orig_buffer[original_counter],None);

                let node2=NodeDynWrap::<N2,T2>::from_ptr_mut(&mut target_buffer[target_counter],Some(node.num));


                (node,node2,size_of_val(node),size_of_val(node))
            }else{
                let node=NodeDstDyn::<N,T>::from_ptr(&orig_buffer[original_counter],None);
                
                let node2=NodeDstDyn::<N2,T2>::from_ptr_mut(&mut target_buffer[target_counter],Some(node.node.num));

                {
                    node2.next_nodes[0]=(node.next_nodes[0]/size_of::<T>())
                    =node.next_nodes;
                    
                }
                node2.comp=node.comp;
                (&node.node,&mut node2.node,size_of_val(node),size_of_val(node))
                //node2.node.num=node.node.num;

            };

            node2.num=node.num;
            node2.dyn.misc=func2(&node.dyn.misc);
            for (a,b) in node.dyn.range.iter().zip(node2.dyn.range.iter_mut()){
                *b=func1(a);
            }

            original_counter+=orig_size;
            target_counter+=target_size;

            leaf_counter!=leaf_counter;
        }
        */
    }
    
    pub fn num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn vistr(&self)->Vistr<N,T>{
        unsafe{
            let buffer=self.mem.get();
            
            let bot_size=std::mem::size_of::<(N,T)>();
            let ptr=&buffer[self.target as usize*bot_size];
            let inner=InnerVistr::new(ptr,self.height);
            Vistr{inner}
        }
    }
    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        unsafe{
            let buffer=self.mem.get_mut();

            let bot_size=std::mem::size_of::<(N,T)>();
            let ptr=&mut buffer[self.target as usize*bot_size];
            let inner=InnerVistrMut::new(ptr,self.height);
            VistrMut{inner}
        }
    }
   
}

fn are_adjacent<'a, T1,T2>(first: &'a [T1], second: &'a [T2]) -> bool {
    let fl = first.len();
    if first[fl..].as_ptr() == second.as_ptr() as *const T1 {
        true
    }else{
        false
    }
}

fn slice_join_mut<'a, T>(first: &'a mut [T], second: &'a mut [T]) -> &'a mut [T] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() == second.as_mut_ptr() {
        unsafe {
            ::std::slice::from_raw_parts_mut(first.as_mut_ptr(), fl + second.len())
        }
    }
    else {
        panic!("Slices not adjacent");
    }
}


fn slice_join_mut2<'a, T>(first: &'a mut [T], second: &'a mut [u8]) -> &'a mut [u8] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() as *mut u8 == second.as_mut_ptr() {
        unsafe {
            ::std::slice::from_raw_parts_mut(first.as_mut_ptr() as *mut u8, fl*std::mem::size_of::<T>() + second.len())
        }
    }
    else {
        panic!("Slices not adjacent");
    }
}

unsafe fn handle_node<T:HasAabb+Copy,N:Copy,S:Sorter,A:AxisTrait,L:LeftOrRight>(sorter:S,axis:A,st:L,bots:&mut [T],buffer:&mut [u8],n:N,height:usize,max_height:usize)->*mut u8
{
    if st.bots_is_right_side_of_buffer(){
        assert!(are_adjacent(buffer,bots));
    }else{
        assert!(are_adjacent(bots,buffer));
    }
    
    let bot_size=std::mem::size_of::<T>();

    if height<max_height-1{
        
        let (fullcomp,left,mid,right)={
                        
            let (fullcomp,left,mid,right)=match construct(sorter,axis,bots){
                Some(pass)=>{
                    pass
                },
                None=>{
                    let mut d=std::mem::uninitialized();
                    std::ptr::write_bytes(&mut d,0,std::mem::size_of::<T::Num>());

                    let empty1:&mut [T]=&mut [];
                    let empty2:&mut [T]=&mut [];
                    let empty3:&mut [T]=&mut [];
                    
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
        

        let shift=NodeDstDyn::<N,T>::empty_size();//number_of_bots_to_cover_non_leaf_node_data::<N,T>();


        let target_med=compute_space::<N,T>(left.len(),height,max_height);
        assert_eq!(target_med % std::mem::align_of::<T>(),0);




        let (lb,left,node,mid,right,rb)=if st.bots_is_right_side_of_buffer(){
            let left_move=target_med-left.len()*bot_size;
            let mid_right_move=target_med+shift;
            let mid_len=mid.len();
            let (a,left,c)=move_bots_left(left,buffer,left_move);

            let (node,mid_right,e)=move_bots_left(slice_join_mut(mid,right),c,mid_right_move);
            
            let (mid,right)=mid_right.split_at_mut(mid_len);
            (a,left,node,mid,right,e)
        }else{
            let mid_right_move=target_med+shift;
            let left_move=target_med-left.len()*bot_size;
            let mid_len=mid.len();
            let (a,mid_right,b)=move_bots_right(slice_join_mut(mid,right),buffer,mid_right_move);
            
            let (c,left,node)=move_bots_right(left,a,left_move);
            
            let (mid,right)=mid_right.split_at_mut(mid_len);
            (c,left,node,mid,right,b)
        };



        
        
        //Construct this node.
        let left_node =handle_node(sorter,axis.next(),RightOf,left,lb,n,height+1,max_height);
        let right_node=handle_node(sorter,axis.next(),LeftOf,right,rb,n,height+1,max_height);

        let node=NodeDstDyn::<N,T>::from_buffer(node,mid.len());

        //TODO add checks
        //let node:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut node[0],size:mid.len()})};
        node.comp=fullcomp;
        node.next_nodes=[left_node,right_node];
        node.node.dyn.misc=n;
        node.node.num=mid.len();
        //Bots should already be in the right spot!

        
    
    
        let ReprMut{ptr,size}=unsafe{std::mem::transmute(node)};
        ptr

    }
    else
    {
        println!("leaf start");
        let shift=NodeDynWrap::<N,T>::empty_size();
        
        
        let (node,bots)=if !st.bots_is_right_side_of_buffer(){
            let (a,bots,_c)=move_bots_right(bots,buffer,shift);
            (a,bots)
            //std::ptr::copy(&mut buffer[0],&mut buffer[shift],bots.len()*bot_size);
        }else{
            (buffer,bots)
        };;

        let node=NodeDynWrap::<N,T>::from_buffer(node,bots.len());
        //let node:&mut NodeDynWrap<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:&mut node[0],size:bots.len()})};
        node.dyn.misc=n;
        node.num=bots.len();
    

        let ReprMut{ptr,size}=unsafe{std::mem::transmute(node)};
        ptr
        

    }



     
}


pub fn move_bots_right<'a,T>(a:&'a mut [T],b:&'a mut [u8],amount:usize)->(&'a mut [u8],&'a mut [T],&'a mut [u8]){
    unsafe{
        let alen=a.len();
        assert!(are_adjacent(a,b));
        let start=a.as_mut_ptr() as *mut u8;
        let end=start.offset(amount as isize);
        assert_eq!(end.align_offset(std::mem::align_of::<T>()),0);


        let target_end=end.offset(alen as isize);
        let blen=b.len();
        let end_ptr=b[blen..].as_mut_ptr();
        assert!(target_end<end_ptr);

        std::ptr::copy(start,end,amount);

        let a=std::slice::from_raw_parts_mut(start,amount);
        let b=std::slice::from_raw_parts_mut(end as *mut T,alen*std::mem::size_of::<T>());
        let crange=(end_ptr as usize-target_end as usize);
        let c=std::slice::from_raw_parts_mut(target_end,crange);
        (a,b,c)
    }
}
pub fn move_bots_left<'a,T>(a:&'a mut [T],b:&'a mut [u8],amount:usize)->(&'a mut [u8],&'a mut [T],&'a mut [u8]){
    assert!(are_adjacent(b,a));
    unimplemented!();
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
