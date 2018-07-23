use super::*;

use compt::CTreeIterator;
use HasAabb;
use std::marker::PhantomData;

pub use self::new::*;


mod new{
    use std::ptr::Unique;
    //TODO give compiler more information about aliasing somehow????????????
  

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


    //This works by inferring the type based on the height.
    //Depending on the height, we will transmute a pointer to this marker type
    //to either a nonleaf or leaf type.
    struct Marker<N,T:HasAabb>(PhantomData<(N,T)>,[u8]);


    use super::*;
    struct NodeDstDyn<N,T:HasAabb>{
        pub next_nodes:(Unique<Marker<N,T>>,Unique<Marker<N,T>>),
        pub comp:FullComp<T::Num>,
        pub node:NodeDyn<N,T>
    }


    /// Tree Iterator that returns a reference to each node.
    /// It also returns the non-leaf specific data when it applies.
    pub struct NdIter<'a,N:'a,T:HasAabb+'a>{
        ptr:&'a Unique<Marker<N,T>>,
        height:usize,
        depth:usize
    }

    impl<'a,N:'a,T:HasAabb+'a> NdIter<'a,N,T>{
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        pub fn create_wrap<'b>(&'b mut self)->NdIter<'b,N,T>{
            NdIter{ptr:self.ptr,height:self.height,depth:self.depth}
        }
    }

    impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIter<'a,N,T>{
        type Item=&'a NodeDyn<N,T>;
        type Extra=Option<&'a FullComp<T::Num>>;
        fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
            let height=self.height;
            if self.depth<self.height-1{
                let node:&'a NodeDstDyn<N,T>=unsafe{std::mem::transmute(*self.ptr)};

                let nn=if node.node.range.is_empty(){
                    None
                }else{
                    Some(&node.comp)
                };

                let stuff=(nn,NdIter{ptr:&node.next_nodes.0,depth:self.depth+1,height},
                NdIter{ptr:&node.next_nodes.1,depth:self.depth+1,height});
                (&node.node,Some(stuff))
            }else{
                let node:&'a NodeDyn<N,T>=unsafe{std::mem::transmute(*self.ptr)};
                (node,None)
            }
            
        }
    }
    
    /// Tree Iterator that returns a reference to each node.
    /// It also returns the non-leaf specific data when it applies.
    pub struct NdIterMut<'a,N:'a,T:HasAabb+'a>{
        ptr:&'a mut Unique<Marker<N,T>>,
        height:usize,
        depth:usize
    }

    impl<'a,N:'a,T:HasAabb+'a> NdIterMut<'a,N,T>{
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        pub fn create_wrap_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
            NdIterMut{ptr:self.ptr,height:self.height,depth:self.depth}
        }
    }

    impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIterMut<'a,N,T>{
        type Item=&'a mut NodeDyn<N,T>;
        type Extra=Option<&'a FullComp<T::Num>>;
        fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
            let height=self.height;
            if self.depth<self.height-1{
                let node:&'a mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(*self.ptr)};

                let nn=if node.node.range.is_empty(){
                    None
                }else{
                    Some(&node.comp)
                };

                let stuff=(nn,NdIterMut{ptr:&mut node.next_nodes.0,depth:self.depth+1,height},
                NdIterMut{ptr:&mut node.next_nodes.1,depth:self.depth+1,height});
                (&mut node.node,Some(stuff))
            }else{
                let node:&'a mut NodeDyn<N,T>=unsafe{std::mem::transmute(*self.ptr)};
                (node,None)
            }
            
        }
    }


    pub struct TreeAllocDstDfsOrder<N,T:HasAabb>{
        _vec:Vec<u8>,
        root:Unique<Marker<N,T>>,
        height:usize
    }

    #[derive(Debug)]
    struct SizRet{
        alignment:usize,
        size_of_non_leaf:usize,
        size_of_leaf:usize,
    }
    impl<N,T:HasAabb> TreeAllocDstDfsOrder<N,T>{

        pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
            NdIterMut{ptr:&mut self.root,depth:0,height:self.height}
        }

        pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
            NdIter{ptr:&self.root,depth:0,height:self.height}
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
            let max_align=alignment.max(alignment2);

            assert_eq!(siz%max_align,0);

            assert_eq!(siz2%max_align,0);

            assert!(std::mem::size_of::<T>() % max_align==0);

            SizRet{alignment:max_align,size_of_non_leaf:siz,size_of_leaf:siz2}
        }


        pub fn new<I:ExactSizeIterator<Item=T>>(it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,height:usize,num_nodes:usize,num_bots:usize)->TreeAllocDstDfsOrder<N,T>{

            let s=Self::compute_alignment_and_size();
            let SizRet{alignment,size_of_non_leaf,size_of_leaf}=s;
            let num_non_leafs=num_nodes/2;
            let num_leafs=num_nodes-num_non_leafs;

            let cap=num_non_leafs*size_of_non_leaf+num_leafs*size_of_leaf+num_bots*std::mem::size_of::<T>();
            

            let (start_addr,vec)={

                let mut v=Vec::with_capacity(alignment+cap);
            
                let mut counter=v.as_ptr() as *mut u8;
     
                let counter={
                    let offset=counter.align_offset(alignment);
                    if offset==usize::max_value(){
                        panic!("Error finding alignment!");
                    }else{
                        unsafe{counter.add(offset)}
                    }
                };

                (unsafe{&mut *counter},v)
            };


            struct Counter{
                //We use a pointer since 
                //as we construct the tree, we create mutable references to then populate the memory
                //that this points to. So this pointer is aliased.
                counter:*mut u8,
                _alignment:usize
            }
            impl Counter{
                fn add_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>)->Unique<NodeDyn<N,T>>{
                    let len=constructor.it.len();
                    let dst:&mut NodeDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                    
                    for (a,b) in dst.range.iter_mut().zip(constructor.it){
                        *a=b;
                    }
                    dst.misc=constructor.misc;

                    self.counter=unsafe{&mut *(self.counter).add(std::mem::size_of_val(dst))};
                    unsafe{Unique::new_unchecked(dst)}
                
                }
                fn add_non_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>,cc:ExtraConstructor<T::Num>)->Unique<NodeDstDyn<N,T>>{
                    let len=constructor.it.len();
                    let dst:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                    
                    for (a,b) in dst.node.range.iter_mut().zip(constructor.it){
                        *a=b;
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

                    self.counter=unsafe{&mut *(self.counter).add(std::mem::size_of_val(dst))};

                    unsafe{Unique::new_unchecked(dst)}
                }
            }

            let mut cc=Counter{_alignment:alignment,counter:start_addr};
            let root=recc(it,&mut cc);
            
            //assert we filled up exactly the amount of space we allocated.
            //Very important assertion!!!!
            assert_eq!(cc.counter as usize,start_addr as *mut u8 as usize+cap);


            return TreeAllocDstDfsOrder{_vec:vec,root:unsafe{Unique::new_unchecked(root)},height};


            fn recc<N,T:HasAabb,I:ExactSizeIterator<Item=T>>
                (it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,counter:&mut Counter)->*mut Marker<N,T>{
                
                let (nn,rest)=it.next();
                
                return match rest{
                    Some((extra,left,right))=>{
                        let left=recc(left,counter);
                         

                        let mut node=counter.add_non_leaf_node(nn,extra);
                        

                        let right=recc(right,counter);
                        
                        let ll=unsafe{Unique::new_unchecked(left)};
                        let rr=unsafe{Unique::new_unchecked(right)};
                        unsafe{node.as_mut()}.next_nodes=(ll,rr);

                    
                        node.as_ptr() as *mut Marker<N,T>
                    },
                    None=>{
                        let mut node=counter.add_leaf_node(nn);
                        
                        node.as_ptr() as *mut Marker<N,T>
                    }
                };   
            }
        }
    }
}

/*


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
unsafe impl<N,T:HasAabb> Sync for NodeDstDyn<N,T>{}


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


#[derive(Debug)]
struct SizRet{
    alignment:usize,
    size_of_non_leaf:usize,
    size_of_leaf:usize,
}
impl<N,T:HasAabb> TreeAllocDstDfsOrder<N,T>{

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
        let max_align=alignment.max(alignment2);

        assert_eq!(siz%max_align,0);

        assert_eq!(siz2%max_align,0);

        assert!(std::mem::size_of::<T>() % max_align==0);

        SizRet{alignment:max_align,size_of_non_leaf:siz,size_of_leaf:siz2}
    }


    pub fn new<I:ExactSizeIterator<Item=T>>(it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,num_nodes:usize,num_bots:usize)->TreeAllocDstDfsOrder<N,T>{

        let s=Self::compute_alignment_and_size();
        let SizRet{alignment,size_of_non_leaf,size_of_leaf}=s;
        let num_non_leafs=num_nodes/2;
        let num_leafs=num_nodes-num_non_leafs;

        let cap=num_non_leafs*size_of_non_leaf+num_leafs*size_of_leaf+num_bots*std::mem::size_of::<T>();
        

        let (start_addr,vec)={

            let mut v=Vec::with_capacity(alignment+cap);
        
            let mut counter=v.as_ptr() as *mut u8;
 
            let counter={
                let offset=counter.align_offset(alignment);
                if offset==usize::max_value(){
                    panic!("Error finding alignment!");
                }else{
                    unsafe{counter.add(offset)}
                }
            };

            (unsafe{&mut *counter},v)
        };


        struct Counter{
            //We use a pointer since 
            //as we construct the tree, we create mutable references to then populate the memory
            //that this points to. So this pointer is aliased.
            counter:*mut u8,
            _alignment:usize
        }
        impl Counter{
            fn add_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>)->std::ptr::NonNull<NodeDyn<N,T>>{
                let len=constructor.it.len();
                let dst:&mut NodeDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                
                for (a,b) in dst.range.iter_mut().zip(constructor.it){
                    *a=b;
                }
                dst.misc=constructor.misc;

                self.counter=unsafe{&mut *(self.counter).add(std::mem::size_of_val(dst))};
                unsafe{std::ptr::NonNull::new_unchecked(dst)}
            
            }
            fn add_non_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>,cc:ExtraConstructor<T::Num>)->std::ptr::NonNull<NodeDstDyn<N,T>>{
                let len=constructor.it.len();
                let dst:&mut NodeDstDyn<N,T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:len})};    
                
                for (a,b) in dst.node.range.iter_mut().zip(constructor.it){
                    *a=b;
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

                self.counter=unsafe{&mut *(self.counter).add(std::mem::size_of_val(dst))};

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
                },
                None=>{
                    let mut node=counter.add_leaf_node(nn);
                    
                    Node2::Leaf(node)
                }
            };   
        }
    }
}
*/