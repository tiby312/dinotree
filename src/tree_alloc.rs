use super::*;

use compt::CTreeIterator;
use HasAabb;
use std::marker::PhantomData;


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



//User provides this!!!!!!!!!!
pub struct ExtraConstructor<N:NumTrait>{
    pub comp:Option<FullComp<N>>
}

pub struct LeafConstructor<N,T:HasAabb,I:ExactSizeIterator<Item=T>>{
    pub misc:N,
    pub it:I
}

///The common struct between leaf nodes and non leaf nodes.
///It is a dynamically sized type.
pub struct NodeDyn<N,T:HasAabb>{
    pub misc:N,
    pub range:[T]
}

///A struct that contains data that only non leaf nodes contain.
#[derive(Copy,Clone)]
pub struct FullComp<N:NumTrait>{
    pub div:N,
    pub cont:axgeom::Range<N> 
}


enum Node2<N,T:HasAabb>{
    Leaf(std::ptr::NonNull<NodeDyn<N,T>>),
    NonLeaf(std::ptr::NonNull<NodeDstDyn<N,T>>)
}


//Unsafely implement sync. even though contains do not impelemnt sync.
//This is safe to do because TODO reason??
unsafe impl<N,T:HasAabb> Sync for Node2<N,T>{}
unsafe impl<N:Send,T:HasAabb+Send> Send for Node2<N,T>{}


impl<N,T:HasAabb> Copy for Node2<N,T>{
}
impl<N,T:HasAabb> Clone for Node2<N,T>{
    fn clone(&self)->Self{
        *self
    }
}

enum NextNodes<N,T:HasAabb>{
    Leaf([std::ptr::NonNull<NodeDyn<N,T>>;2]),
    NonLeaf([std::ptr::NonNull<NodeDstDyn<N,T>>;2])
}

struct NodeDstDyn<N,T:HasAabb>{
    pub next_nodes:NextNodes<N,T>,
    pub comp:FullComp<T::Num>,
    pub node:NodeDyn<N,T>
}

unsafe impl<N:Send,T:HasAabb+Send> Send for NodeDstDyn<N,T>{}
//unsafe impl<N:Sync,T:HasAabb+Sync> Sync for NodeDstDyn<N,T>{}
//Unsafely implement sync. even though contains do not impelemnt sync.
//This is safe to do because TODO reason??
unsafe impl<N,T:HasAabb> Sync for NodeDstDyn<N,T>{}


pub use self::det::NdIterMove;
mod det{
    use super::*;

    pub struct LeafRangeDestructor<T>{
        inner:*mut [T],
        count:usize
    }
    impl<T> Drop for LeafRangeDestructor<T>{
        fn drop(&mut self){
            //Call destructor for all bots that were not consumed.
            for _b in self{}
        }
    }
    impl<T> ExactSizeIterator for LeafRangeDestructor<T>{

    }
    impl<T> Iterator for LeafRangeDestructor<T>{
        type Item=T;
        fn next(&mut self)->Option<T>{
            let inner=unsafe{&mut *self.inner};

            if self.count>=inner.len(){
                None
            }else{
                //TODO remove boundscheck
                let b=&mut inner[self.count];
                self.count+=1;
                let mut bot=unsafe{std::mem::uninitialized()};
                unsafe{std::ptr::copy(b,&mut bot,1)};
                    
                Some(bot)
            }
        }
        fn size_hint(&self)->(usize,Option<usize>){
            let l=unsafe{(*self.inner).len()};
            (l,Some(l))
        }
    }

    //TODO future optimization: dont dynamically allocate this shared object.
    pub(super) struct Shared{
        pub vec:Vec<u8>,
    }

    ///A Tree Iterator that moves the contents of all the nodes out and to the user.
    ///When you turn a Vec<T> into an iterator, it is obvious to know
    ///at what point all the elements are consumed and it is safe to deallocate the underlying memory.
    ///With a tree, it is much harder. The user might consume all the elements in any order that they choose.
    ///Reference counting is therefore used. A shared counter keeps track of how many nodes have been consumed.
    ///Only when all the nodes have been consumed do we deallocate the tree.
    ///Note that send and sync are not implemented so that the shared reference count 
    ///has no overhead of a mutex without the obvious downside of loss in parallelism.
    pub struct NdIterMove<N,T:HasAabb>(Option<NdIterMoveInner<N,T>>,Rc<Shared>);

    //When a node dies without being consumed, we have to recurse all the children and destroy them.
    impl<N,T:HasAabb> Drop for NdIterMove<N,T>{
        fn drop(&mut self){
            if self.0.is_some(){
                for _b in self.0.take().unwrap().dfs_preorder_iter(){}
            }
        }
    }
    impl<N,T:HasAabb> NdIterMove<N,T>{
        pub(super) fn new(a:Node2<N,T>,shared:Rc<Shared>)->Self{
            NdIterMove(Some(NdIterMoveInner(a)),shared)
        }
    }

    impl<N,T:HasAabb> CTreeIterator for NdIterMove<N,T>{
        type Item=(N,LeafRangeDestructor<T>);
        type Extra=Option<FullComp<T::Num>>;
        fn next(mut self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
            let (nn,rest)=self.0.take().unwrap().next();

            (nn,match rest{
                None=>{
                    None
                },
                Some((extra,left,right))=>{
                    
                    Some((extra,NdIterMove(Some(left),self.1.clone()),NdIterMove(Some(right),self.1.clone())))
                }
            })
        }
    }

    struct NdIterMoveInner<N,T:HasAabb>(
        Node2<N,T>    
    );



    unsafe fn copy_unsafe<T>(bla:&T)->T{
        let mut b=std::mem::uninitialized();
        std::ptr::copy(bla,&mut b,1);
        b
    }
    impl<N,T:HasAabb> CTreeIterator for NdIterMoveInner<N,T>{
        type Item=(N,LeafRangeDestructor<T>);
        type Extra=Option<FullComp<T::Num>>;
        fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
            match self.0{
                Node2::Leaf(mut leaf)=>{
                    let leaf=unsafe{leaf.as_mut()};
                    let range=LeafRangeDestructor{inner:&mut leaf.range,count:0};


                    ((unsafe{copy_unsafe(&leaf.misc)},range),None)
                },
                Node2::NonLeaf(mut nonleaf)=>{
                    let nonleaf=unsafe{nonleaf.as_mut()};
                    let [left,right]=match nonleaf.next_nodes{
                        NextNodes::Leaf([left,right])=>{
                            [Node2::Leaf(left),Node2::Leaf(right)]
                        },
                        NextNodes::NonLeaf([left,right])=>{
                            [Node2::NonLeaf(left),Node2::NonLeaf(right)]
                        }
                    };
                    
                    let [left,right]=[NdIterMoveInner(left),NdIterMoveInner(right)];

                    let rr=if nonleaf.node.range.is_empty(){
                        None
                    }else{
                        Some(nonleaf.comp)
                    };

                    let nn=LeafRangeDestructor{inner:&mut nonleaf.node.range,count:0};
                    
                    ((unsafe{copy_unsafe(&nonleaf.node.misc)},nn),Some((rr,left,right)))
                }
            }   
        }
    }

}

///Tree Iterator that returns a mutable reference to each node.
///It also returns the non-leaf specific data when it applies.
///It is important that while the user can get mutable references to the bots
///using this iterator, the user does not modify the aabb() that the bots return.
///This would invalid the invariants of the tree.
pub struct NdIterMut<'a,N:'a,T:HasAabb+'a>(
    (Node2<N,T>,PhantomData<&'a mut usize>)
);
impl<'a,N:'a,T:HasAabb+'a> NdIterMut<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        NdIterMut(((self.0).0,PhantomData))
    }
}

impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIterMut<'a,N,T>{
    type Item=&'a mut NodeDyn<N,T>;
    type Extra=Option<&'a FullComp<T::Num>>;
    fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
        match (self.0).0{
            Node2::Leaf(leaf)=>{
                let leaf=unsafe{&mut *leaf.as_ptr()};
                (leaf,None)
            },
            Node2::NonLeaf(nonleaf)=>{
                let nonleaf=unsafe{&mut *nonleaf.as_ptr()};
                let [left,right]=match nonleaf.next_nodes{
                    NextNodes::Leaf([left,right])=>{
                        [Node2::Leaf(left),Node2::Leaf(right)]
                    },
                    NextNodes::NonLeaf([left,right])=>{
                        [Node2::NonLeaf(left),Node2::NonLeaf(right)]
                    }
                };
                let [left,right]=[NdIterMut((left,PhantomData)),NdIterMut((right,PhantomData))];

                let rr=if nonleaf.node.range.is_empty(){
                    None
                }else{
                    Some(&nonleaf.comp)
                };
                (&mut nonleaf.node,Some((rr,left,right)))
            }
        }
    }
}

/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct NdIter<'a,N:'a,T:HasAabb+'a>(
    (Node2<N,T>,PhantomData<&'a usize>)
);

impl<'a,N:'a,T:HasAabb+'a> NdIter<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b mut self)->NdIter<'b,N,T>{
        NdIter(((self.0).0,PhantomData))
    }
}

impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIter<'a,N,T>{
    type Item=&'a NodeDyn<N,T>;
    type Extra=Option<&'a FullComp<T::Num>>;
    fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
        match (self.0).0{
            Node2::Leaf(leaf)=>{
                let leaf=unsafe{&mut *leaf.as_ptr()};
                (leaf,None)
            },
            Node2::NonLeaf(nonleaf)=>{
                let nonleaf=unsafe{&mut *nonleaf.as_ptr()};
                let [left,right]=match nonleaf.next_nodes{
                    NextNodes::Leaf([left,right])=>{
                        [Node2::Leaf(left),Node2::Leaf(right)]
                    },
                    NextNodes::NonLeaf([left,right])=>{
                        [Node2::NonLeaf(left),Node2::NonLeaf(right)]
                    }
                };
                let [left,right]=[NdIter((left,PhantomData)),NdIter((right,PhantomData))];


                let rr=if nonleaf.node.range.is_empty(){
                    None
                }else{
                    Some(&nonleaf.comp)
                };

                (& nonleaf.node,Some((rr,left,right)))
            }
        }
    }
}


pub struct TreeAllocDstDfsOrder<N,T:HasAabb>{
    _vec:Option<Vec<u8>>,
    root:Node2<N,T>
}

unsafe impl<N:Send,T:HasAabb+Send> Send for TreeAllocDstDfsOrder<N,T>{}
unsafe impl<N:Sync,T:HasAabb+Sync> Sync for TreeAllocDstDfsOrder<N,T>{}
use std::rc::Rc;



impl<N,T:HasAabb> Drop for TreeAllocDstDfsOrder<N,T>{
    fn drop(&mut self){
        match self._vec.take(){
            None=>{
                //We already move out the vec using the iterator.
            },
            Some(v)=>{
                //TODO when dropping, we can establish a order of traversal of the tree.
                //so we DONT NEED reference counting. TODO make a version that simply frees the 
                //resource after all the nodes have been visited in dfs preorder.
                let shared=Rc::new(det::Shared{vec:v});
                {
                    let cc=shared.clone();
                    let it=self::det::NdIterMove::new(self.root,cc);
                    for _b in it.dfs_preorder_iter(){}
                }
                assert!(Rc::strong_count(&shared)==1);
                assert!(Rc::weak_count(&shared)==0);
            }
        }
       
    }
}

#[derive(Debug)]
struct SizRet{
    alignment:usize,
    size_of_non_leaf:usize,
    size_of_leaf:usize,
}
impl<N,T:HasAabb> TreeAllocDstDfsOrder<N,T>{

    pub fn into_iterr(mut self)->self::det::NdIterMove<N,T>{
        let shared=Rc::new(det::Shared{vec:self._vec.take().unwrap()});
        self::det::NdIterMove::new(self.root,shared)
    }

    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        NdIterMut((self.root,PhantomData))
    }

    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        NdIter((self.root,PhantomData))
    }

    //fn compute_alignment_and_size()->(usize,usize){
    fn compute_alignment_and_size()->SizRet{  
        let (alignment,siz)={
            let k:&NodeDstDyn<N,T>=unsafe{

                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };

        let (alignment2,siz2)={
            let k:&NodeDyn<N,T>=unsafe{

                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };
        //assert_eq!(alignment,alignment2);
        let max_align=alignment.max(alignment2);

        assert_eq!(siz%max_align,0);

        assert_eq!(siz2%max_align,0);

        assert!(std::mem::size_of::<T>() % max_align==0);

        SizRet{alignment:max_align,size_of_non_leaf:siz,size_of_leaf:siz2}
        //(max_align,siz)
    }


    pub fn new<I:ExactSizeIterator<Item=T>>(it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,num_nodes:usize,num_bots:usize)->TreeAllocDstDfsOrder<N,T>{

        let s=Self::compute_alignment_and_size();
        //println!("Size ret={:?}",s);
        let SizRet{alignment,size_of_non_leaf,size_of_leaf}=s;
        let num_non_leafs=num_nodes/2;
        let num_leafs=num_nodes-num_non_leafs;

        let cap=num_non_leafs*size_of_non_leaf+num_leafs*size_of_leaf+num_bots*std::mem::size_of::<T>();
        //let cap=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        //let cap=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        

        let (start_addr,vec)={

            let mut v=Vec::with_capacity(alignment+cap);
        
            let mut counter=v.as_ptr() as *mut u8;
 

            for _ in 0..alignment{
                let k=counter as *const u8;
                if k as usize % alignment == 0{
                    break;
                }
                counter=unsafe{counter.offset(1)};
            } 
            (unsafe{&mut *counter},v)
        };


        struct Counter{
            counter:*mut u8,
            _alignment:usize
        }
        impl Counter{
            fn add_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>)->std::ptr::NonNull<NodeDyn<N,T>>{
                let len=constructor.it.len();
                let dst:&mut NodeDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                
                //UNSAFE!!!!!!!Leave next nodes uninitialized
                //dst.c=None; //We set the children later
                for (a,b) in dst.range.iter_mut().zip(constructor.it){
                    unsafe{std::ptr::copy(&b,a,1)};
                }
                dst.misc=constructor.misc;

                //func(stuff.1,dst,None);
                self.counter=unsafe{&mut *(self.counter).offset(std::mem::size_of_val(dst) as isize)};
                //assert_eq!(self.counter as usize % self.alignment,0);
                unsafe{std::ptr::NonNull::new_unchecked(dst)}
            
            }
            fn add_non_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>,cc:ExtraConstructor<T::Num>)->std::ptr::NonNull<NodeDstDyn<N,T>>{
                let len=constructor.it.len();
                let dst:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                
                //UNSAFE!!!!!!!Leave next nodes uninitialized
                //dst.c=None; //We set the children later
                for (a,b) in dst.node.range.iter_mut().zip(constructor.it){
                    unsafe{std::ptr::copy(&b,a,1)};
                }
                dst.node.misc=constructor.misc;

                match cc.comp{
                    Some(comp)=>{
                        dst.comp=comp;
                    },
                    None=>{
                        //Leav uninitailized.
                        //and make sure the length is zero so it is never accessed
                        assert!(len==0);
                    }
                }

                self.counter=unsafe{&mut *(self.counter).offset(std::mem::size_of_val(dst) as isize)};
                //assert_eq!(self.counter as usize % self.alignment,0);
                unsafe{std::ptr::NonNull::new_unchecked(dst)}
            }
        }

        let mut cc=Counter{_alignment:alignment,counter:start_addr};
        let root=recc(it,&mut cc);
        
        //assert we filled up exactly the amount of space we allocated.
        //Very important assertion!!!!
        assert_eq!(cc.counter as usize,start_addr as *mut u8 as usize+cap);


        return TreeAllocDstDfsOrder{_vec:Some(vec),root};


        fn recc<N,T:HasAabb,I:ExactSizeIterator<Item=T>>
            (it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,counter:&mut Counter)->Node2<N,T>{
            
            let (nn,rest)=it.next();
            
            return match rest{
                Some((extra,left,right))=>{
                    let left=recc(left,counter);
                    

                    let mut node=counter.add_non_leaf_node(nn,extra);
                    {
                        let node=unsafe{node.as_mut()};

                        let right=recc(right,counter);
                        
                        match (left,right){
                            (Node2::Leaf(left),Node2::Leaf(right))=>{

                                node.next_nodes=NextNodes::Leaf([left,right]);
                            },
                            (Node2::NonLeaf(left),Node2::NonLeaf(right))=>{

                                node.next_nodes=NextNodes::NonLeaf([left,right]);
                            },
                            _=>{unreachable!()}
                        }
                    }
                    Node2::NonLeaf(node)
                    //Do stuff here! Now both children okay
                },
                None=>{
                    let mut node=counter.add_leaf_node(nn);
                    
                    Node2::Leaf(node)
                }
            };   
        }


    }
    /*
    pub fn new<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<N,T>,Option<&mut FullComp<T::Num>>)>(
            num_nodes:usize,num_bots:usize,it:C,func:F)->TreeAllocDstDfsOrder<N,T>{


        let (alignment,node_size)=Self::compute_alignment_and_size();

        let cap=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        
        let (start_addr,vec)={

            let mut v=Vec::with_capacity(alignment+cap);
        
            v.push(0);
            let mut counter=(&mut v[0]) as *mut u8;
            v.pop();
            

            for _ in 0..alignment{
                let k=counter as *const u8;
                if k as usize % alignment == 0{
                    break;
                }
                counter=unsafe{counter.offset(1)};
            } 
            (unsafe{&mut *counter},v)
        };

        
        struct Counter{
            counter:*mut u8
        }
        impl Counter{
            fn add_leaf_node<N,T:HasAabb,B,F:Fn(B,&mut NodeDyn<N,T>,Option<&mut FullComp<T::Num>>)>(&mut self,stuff:(usize,B),func:&F)->*mut NodeDyn<N,T>{
                let dst:&mut NodeDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:stuff.0})};    
                
                //UNSAFE!!!!!!!Leave next nodes uninitialized
                //dst.c=None; //We set the children later
                func(stuff.1,dst,None);
                self.counter=unsafe{&mut *(self.counter).offset(std::mem::size_of_val(dst) as isize)};
                dst
            
            }
            fn add_non_leaf_node<N,T:HasAabb,B,F:Fn(B,&mut NodeDyn<N,T>,Option<&mut FullComp<T::Num>>)>(&mut self,stuff:(usize,B),func:&F)->*mut NodeDstDyn<N,T>{
                let dst:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:stuff.0})};    
                
                //UNSAFE!!!!!!!Leave next nodes uninitialized
                //dst.c=None; //We set the children later
                func(stuff.1,&mut dst.n.node,Some(&mut dst.n.comp));
                self.counter=unsafe{&mut *(self.counter).offset(std::mem::size_of_val(dst) as isize)};
                dst
            }
        }

        let mut cc=Counter{counter:start_addr};
        let root=recc(it,&func,&mut cc);
        
        return TreeAllocDstDfsOrder{_vec:vec,root};


        fn recc<N,T:HasAabb,B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<N,T>,Option<&mut FullComp<T::Num>>)>
            (it:C,func:&F,counter:&mut Counter)->Node2<N,T>{
            
            let (nn,rest)=it.next();
            
            return match rest{
                Some((extra,left,right))=>{
                    let left=recc(left,func,counter);
                    

                    let mut node=counter.add_non_leaf_node(nn,func);
                    {
                        let node=unsafe{&mut *node};

                        let right=recc(right,func,counter);
                        
                        match (left,right){
                            (Node2::Leaf(left),Node2::Leaf(right))=>{

                                node.next_nodes=NextNodes::Leaf([left,right]);
                            },
                            (Node2::NonLeaf(left),Node2::NonLeaf(right))=>{

                                node.next_nodes=NextNodes::NonLeaf([left,right]);
                            },
                            _=>{unreachable!()}
                        }
                    }
                    Node2::NonLeaf(node)
                    //Do stuff here! Now both children okay
                },
                None=>{
                    let mut node=counter.add_leaf_node(nn,func);
                    
                    Node2::Leaf(node)
                }
            };   
        }
    }
    */
}

/*
pub struct TreeAllocDst<T:HasAabb>{
    _vec:Vec<u8>,
    root:*mut NodeDstDyn<T>
}

impl<T:HasAabb> TreeAllocDst<T>{   

    pub fn get_root_mut(&mut self)->&mut NodeDstDyn<T>{
        unsafe{std::mem::transmute(self.root)}
    }

    pub fn get_root(&self)->&NodeDstDyn<T>{
        unsafe{std::mem::transmute(self.root)}
    }

    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,T>{
        NdIterMut{c:self.get_root_mut()}
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,T>{
        NdIter{c:self.get_root()}
    }

    pub fn new<II:Iterator<Item=T>,I:CTreeIterator<Item=NodeDynBuilder<II,T>>>(num_nodes:usize,num_bots:usize,it:I)->TreeAllocDst<T>{
        let mut it=it.bfs_iter();

        let (alignment,node_size)=Self::compute_alignment_and_size();

        let cap=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        
        let (start_addr,vec)={

            let mut v=Vec::with_capacity(alignment+cap);
        
            v.push(0);
            let mut counter=(&mut v[0]) as *mut u8;
            v.pop();
            

            for _ in 0..alignment{
                let k=counter as *const u8;
                if k as usize % alignment == 0{
                    break;
                }
                counter=unsafe{counter.offset(1)};
            } 
            (counter,v)
        };

        let max_counter=unsafe{start_addr.offset(cap as isize)};

       

        let mut queue:Vec<&mut NodeDstDyn<T>>=Vec::with_capacity(num_nodes);
        
        let mut counter=start_addr;
        for builder in it{

            let dst={
                let dst:&mut NodeDstDyn<T>=unsafe{std::mem::transmute(ReprMut{ptr:counter,size:builder.num_bots})};    
                dst.c=None; //We set the children later
                dst.n.divider=builder.divider;
                dst.n.container_box=builder.container_box;

                for (a,b) in dst.n.range.iter_mut().zip(builder.range){
                    //let k=&mut all_bots[b.index as usize];
                    //we cant just move it into here.
                    //then rust will try and call the destructor of the uninitialized object
                    unsafe{std::ptr::copy(&b,a,1)};
                    std::mem::forget(b);
                }
                dst
            };
            counter=unsafe{counter.offset(std::mem::size_of_val(dst) as isize)};
       
            queue.push(dst);

        }
        assert!( counter as *const u8== max_counter);
        assert_eq!(queue.len(),num_nodes);
     
        for i in (1..(num_nodes/2)+1).rev(){
            let c2=queue.pop().unwrap();
            let c1=queue.pop().unwrap();
            let j=2*i;
            let parent=(j-1)/2;
            queue[parent].c=Some((c1,c2)); 
        }

        assert_eq!(queue.len(),1);
        let root=queue.pop().unwrap();
        let root=unsafe{std::mem::transmute(root)};
        TreeAllocDst{_vec:vec,root:root}
    }


    fn compute_alignment_and_size()->(usize,usize){
        
        let (alignment,siz)={
            let k:&NodeDstDyn<T>=unsafe{
            //let mut vec:Vec<u8>=Vec::with_capacity(500);
            //vec.push(0);
            //let x:&[u8]= std::slice::from_raw_parts(&vec[0], 200+std::mem::size_of::<T>()); 
            
                let k:*const u8=std::mem::transmute(0x10 as usize);//std::ptr::null::<T>();
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };

        assert!(std::mem::size_of::<T>() % alignment==0);

        (alignment,siz)
    }

}
*/