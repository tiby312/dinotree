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



/*
//TODO use???
pub struct LeafDyn<N,T:HasAabb>{
    pub misc:N,
    pub range:[T]
}
*/
/*
pub struct NodeDyn<N,T:HasAabb>{ 
    //Carry some user defined data.
    //Useful for nbody simulation
    pub misc:N,

    //TODO explain that these are private since they only apply to non leafs.
    //type system protects against this
    //For non leaf:
    //div is None iff this node and children nodes do not have any bots in them.
    //For leaf:
    //Always none
    //pub(crate) div:Option<(T::Num,axgeom::Range<T::Num>)>,
 //If this is a non leaf node, then,
    //  div is None iff this node and children nodes do not have any bots in them.
    // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
    // This is because we specifically pick the median.



    //For non leaf:
    //cont is None iff range.len()==0
    //For leaf:
    //cont is always none.
    //pub(crate) cont:Option<axgeom::Range<T::Num>>,
    //TODO combine this option with the above one.
    //It is guarenteed that there be at least one bot in this node if the divider exists.

    
    //The range field acts as a sentinel.
    //These two values are only valid if the length of range is greater than zero.
    //For leafs, even if the range is not zero it is still unitiailized
    pub(crate) div:(T::Num,axgeom::Range<T::Num>),

    pub range:[T]

}
*/

//User provides this!!!!!!!!!!
pub struct ExtraConstructor<N:NumTrait>{
    pub comp:Option<(N,axgeom::Range<N>)>
}

pub struct LeafConstructor<N,T:HasAabb,I:ExactSizeIterator<Item=T>>{
    pub misc:N,
    pub it:I
}



pub struct NodeDyn<N,T:HasAabb>{
    pub misc:N,
    pub range:[T]
}

#[derive(Copy,Clone)]
pub struct FullComp<N:NumTrait>{
    pub div:(N,axgeom::Range<N>) 
}


pub enum Node2<N,T:HasAabb>{
    Leaf(*mut NodeDyn<N,T>),
    NonLeaf(*mut NodeDstDyn<N,T>)
}


unsafe impl<N:Send,T:HasAabb+Send> Send for Node2<N,T>{}
//unsafe impl<N:Sync,T:HasAabb+Sync> Sync for NodeDstDyn<N,T>{}
//Unsafely implement sync. even though contains do not impelemnt sync.
//This is safe to do because TODO reason??
unsafe impl<N,T:HasAabb> Sync for Node2<N,T>{}


impl<N,T:HasAabb> Copy for Node2<N,T>{
}
impl<N,T:HasAabb> Clone for Node2<N,T>{
    fn clone(&self)->Self{
        *self
    }
}

pub enum NextNodes<N,T:HasAabb>{
    Leaf([*mut NodeDyn<N,T>;2]),
    NonLeaf([*mut NodeDstDyn<N,T>;2])
}


pub struct NodeDstDyn<N,T:HasAabb>{
    pub next_nodes:NextNodes<N,T>,
    pub comp:FullComp<T::Num>,
    pub node:NodeDyn<N,T>
}

unsafe impl<N:Send,T:HasAabb+Send> Send for NodeDstDyn<N,T>{}
//unsafe impl<N:Sync,T:HasAabb+Sync> Sync for NodeDstDyn<N,T>{}
//Unsafely implement sync. even though contains do not impelemnt sync.
//This is safe to do because TODO reason??
unsafe impl<N,T:HasAabb> Sync for NodeDstDyn<N,T>{}



use std::cell::RefCell;
pub mod det{
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
        type Extra=Option<(T::Num,axgeom::Range<T::Num>)>;
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
        type Extra=Option<(T::Num,axgeom::Range<T::Num>)>;
        fn next(mut self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
            match self.0{
                Node2::Leaf(leaf)=>{
                    let leaf=unsafe{&mut *leaf};
                    let range=LeafRangeDestructor{inner:&mut leaf.range,count:0};


                    ((unsafe{copy_unsafe(&leaf.misc)},range),None)
                },
                Node2::NonLeaf(nonleaf)=>{
                    let nonleaf=unsafe{&mut *nonleaf};
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
                        Some(nonleaf.comp.div)
                    };

                    let nn=LeafRangeDestructor{inner:&mut nonleaf.node.range,count:0};
                    
                    ((unsafe{copy_unsafe(&nonleaf.node.misc)},nn),Some((rr,left,right)))
                }
            }   
        }
    }

}


pub struct NdIterMut<'a,N:'a,T:HasAabb+'a>(
    (Node2<N,T>,PhantomData<&'a mut usize>)
);
impl<'a,N:'a,T:HasAabb+'a> NdIterMut<'a,N,T>{
    pub fn create_wrap_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        NdIterMut(((self.0).0,PhantomData))
    }
}


impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIterMut<'a,N,T>{
    type Item=&'a mut NodeDyn<N,T>;
    type Extra=Option<(T::Num,axgeom::Range<T::Num>)>;
    fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
        match (self.0).0{
            Node2::Leaf(leaf)=>{
                let leaf=unsafe{&mut *leaf};
                (leaf,None)
            },
            Node2::NonLeaf(nonleaf)=>{
                let nonleaf=unsafe{&mut *nonleaf};
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
                    Some(nonleaf.comp.div)
                };
                (&mut nonleaf.node,Some((rr,left,right)))
            }
        }
    }
}



pub struct NdIter<'a,N:'a,T:HasAabb+'a>(
    (Node2<N,T>,PhantomData<&'a usize>)
);

impl<'a,N:'a,T:HasAabb+'a> NdIter<'a,N,T>{
    pub fn create_wrap<'b>(&'b mut self)->NdIter<'b,N,T>{
        NdIter(((self.0).0,PhantomData))
    }
}

impl<'a,N:'a,T:HasAabb+'a> CTreeIterator for NdIter<'a,N,T>{
    type Item=&'a NodeDyn<N,T>;
    type Extra=Option<(T::Num,axgeom::Range<T::Num>)>;
    fn next(self)->(Self::Item,Option<(Self::Extra,Self,Self)>){
        match (self.0).0{
            Node2::Leaf(leaf)=>{
                let leaf=unsafe{&mut *leaf};
                (leaf,None)
            },
            Node2::NonLeaf(nonleaf)=>{
                let nonleaf=unsafe{&mut *nonleaf};
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
                    Some(nonleaf.comp.div)
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



    fn compute_alignment_and_size()->(usize,usize){
        
        let (alignment,siz)={
            let k:&NodeDstDyn<N,T>=unsafe{

                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };

        assert!(std::mem::size_of::<T>() % alignment==0);

        (alignment,siz)
    }


    pub fn new<I:ExactSizeIterator<Item=T>>(it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,num_nodes:usize,num_bots:usize)->TreeAllocDstDfsOrder<N,T>{

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
            fn add_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>)->*mut NodeDyn<N,T>{
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
                dst
            
            }
            fn add_non_leaf_node<N,T:HasAabb,I:ExactSizeIterator<Item=T>>(&mut self,constructor:LeafConstructor<N,T,I>,cc:ExtraConstructor<T::Num>)->*mut NodeDstDyn<N,T>{
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
                        dst.comp=FullComp{div:comp};
                    },
                    None=>{
                        //Leav uninitailized.
                        //and make sure the length is zero so it is never accessed
                        assert!(len==0);
                    }
                }

                self.counter=unsafe{&mut *(self.counter).offset(std::mem::size_of_val(dst) as isize)};
                dst
            }
        }

        let mut cc=Counter{counter:start_addr};
        let root=recc(it,&mut cc);
        
        return TreeAllocDstDfsOrder{_vec:Some(vec),root};


        fn recc<N,T:HasAabb,I:ExactSizeIterator<Item=T>>
            (it:impl CTreeIterator<Item=LeafConstructor<N,T,I>,Extra=ExtraConstructor<T::Num>>,counter:&mut Counter)->Node2<N,T>{
            
            let (nn,rest)=it.next();
            
            return match rest{
                Some((extra,left,right))=>{
                    let left=recc(left,counter);
                    

                    let mut node=counter.add_non_leaf_node(nn,extra);
                    {
                        let node=unsafe{&mut *node};

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