use super::*;

use compt::Visitor;
use HasAabb;
use std::marker::PhantomData;
use std::iter::TrustedLen;
use inner_prelude::*;
use tools::*;
use dinotree_inner::Sorter;



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


pub struct NodeDstDyn<N,T:HasAabb>{
    //This term can't live in fullcomp, since every if there are no bots in a node, or below,
    //we might want to traverse the lower children to construct the tree properly.
    pub next_nodes:[i32;2], //offset from parents in terms of bytes   //TODO change these to i32!!!!!!!!!!!!!

    pub comp:FullComp<T::Num>,
        
    pub node:NodeDynWrap<N,T>
}

impl<N,T:HasAabb> NodeDstDyn<N,T>{
    pub fn as_ptr(&self)->*const u8{
        let alloc::Repr{ptr,size}=unsafe{std::mem::transmute(self)};
        ptr
    }
}

pub struct NodeDynWrap<N,T>{
    pub num:i32, //TODO hcange these to i32
    pub dyn:NodeDyn<N,T>
}

impl<N,T:HasAabb> NodeDynWrap<N,T>{
    pub fn as_ptr(&self)->*const u8{
        let alloc::Repr{ptr,size}=unsafe{std::mem::transmute(self)};
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
    pub fn new(root:&'a u8,height:usize)->Vistr<'a,N,T>{
        unimplemented!()
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->Vistr<'b,N,T>{
        //Vistr{inner:self.inner.create_wrap()}
        unimplemented!()
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
        unimplemented!()
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        unimplemented!()
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
    pub fn new(root:&'a mut u8,height:usize)->VistrMut<'a,N,T>{
        unimplemented!()
    }
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        unimplemented!()
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
        unimplemented!()
        
    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        unimplemented!()
    }
}



/*

pub struct InnerVistrMut<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a mut u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a mut (T,N)>
}
impl<'a,N:'a,T:HasAabb+'a> InnerVistrMut<'a,N,T>{
    pub fn new(ptr:&'a mut u8,max_height:usize)->InnerVistrMut<'a,N,T>{
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
    type NonLeafItem=(usize,[usize;2],&'a FullComp<T::Num>);
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                
                /*
                let node= NodeDstDyn::<N,T>::from_ptr_mut(self.ptr,None);

                let nn=(self.height,node.next_nodes,&node.comp);

                let left_pointer:&'a mut u8=std::mem::transmute(node.next_nodes[0]);
                let right_pointer:&'a mut u8=std::mem::transmute(node.next_nodes[1]);

                let a=InnerVistrMut{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistrMut{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                (&mut node.node,Some((nn,a,b)))
                */
                unimplemented!()
            }else{
                /*
                let node=NodeDynWrap::<N,T>::from_ptr_mut(self.ptr,None);
                (node,None)
                */
                unimplemented!()
            }
        }
    }

    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}


pub struct InnerVistr<'a,N:'a,T:HasAabb+'a>{
    ptr:&'a u8,
    height:usize,
    depth:usize,
    _p:PhantomData<&'a (T,N)>
}
impl<'a,N:'a,T:HasAabb+'a> InnerVistr<'a,N,T>{
    pub fn new(ptr:&'a u8,max_height:usize)->InnerVistr<'a,N,T>{
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
    type NonLeafItem=(usize,[usize;2],&'a FullComp<T::Num>);
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        unsafe{
            let height=self.height;
            if self.depth<self.height-1{
                /*
                let node=NodeDstDyn::<N,T>::from_ptr(self.ptr,None);

                let nn=(self.height,node.next_nodes,&node.comp);


                let left_pointer:&'a u8=std::mem::transmute(node.next_nodes[0]);
                let right_pointer:&'a u8=std::mem::transmute(node.next_nodes[1]);

                let a=InnerVistr{ptr:left_pointer,depth:self.depth+1,height,_p:PhantomData};
                let b=InnerVistr{ptr:right_pointer,depth:self.depth+1,height,_p:PhantomData};

                (&node.node,Some((nn,a,b)))
                */
                unimplemented!()
            }else{
                /*
                let node=NodeDynWrap::<N,T>::from_ptr(self.ptr,None);
                (node,None)
                */
                unimplemented!()
            }
        }
    }

    fn level_remaining_hint(&self)->(usize,Option<usize>){
        let d=self.height-self.depth;
        (d,Some(d))
    }
}

*/






