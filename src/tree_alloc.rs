use super::*;

use compt::CTreeIterator;
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




pub struct NodeDyn<T:SweepTrait>{ 

    pub divider:T::Num,

    //only valid if the node has bots in it.
    pub container_box:axgeom::Range<T::Num>,
    
    pub range:[T]
}


pub struct NdIterMut<'a,T:SweepTrait+'a>{
    c:&'a mut NodeDstDyn<T>
}

impl<'a,T:SweepTrait+'a> CTreeIterator for NdIterMut<'a,T>{
    type Item=&'a mut NodeDyn<T>;
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let i=&mut self.c.n;
        let o=match self.c.c{
            Some((mut a,mut b))=>{
                let a=unsafe{&mut *a};
                let b=unsafe{&mut *b};
                Some((NdIterMut{c:a},NdIterMut{c:b}))
            },
            None=>{
                None
            }
        };
        (i,o)
    }
}

pub struct NdIter<'a,T:SweepTrait+'a>{
    c:&'a NodeDstDyn<T>
}

impl<'a,T:SweepTrait+'a> CTreeIterator for NdIter<'a,T>{
    type Item=&'a NodeDyn<T>;
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let i=&self.c.n;
        let o=match self.c.c{
            Some(( a, b))=>{
                let a=unsafe{& *a};
                let b=unsafe{& *b};
                Some((NdIter{c:a},NdIter{c:b}))
            },
            None=>{
                None
            }
        };
        (i,o)
    }
}



pub struct NodeDstDyn<T:SweepTrait>{
    c:Option<(*mut NodeDstDyn<T>,*mut NodeDstDyn<T>)>,
    pub n:NodeDyn<T>
}
unsafe impl<T:SweepTrait> Send for NodeDstDyn<T>{}





pub struct TreeAllocDstDfsOrder<T:SweepTrait>{
    _vec:Vec<u8>,
    root:*mut NodeDstDyn<T>
}


impl<T:SweepTrait> TreeAllocDstDfsOrder<T>{
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



    fn compute_alignment_and_size()->(usize,usize){
        
        let (alignment,siz)={
            let k:&NodeDstDyn<T>=unsafe{

                let k:*const u8=std::mem::transmute(0x10 as usize);
                std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };

        assert!(std::mem::size_of::<T>() % alignment==0);

        (alignment,siz)
    }


    pub fn new<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<T>)>(
        num_nodes:usize,num_bots:usize,it:C,func:F)->TreeAllocDstDfsOrder<T>{
        
        Self::new_inner(num_nodes,num_bots,it,func)
    }

    pub fn new_inner<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<T>)>(
            num_nodes:usize,num_bots:usize,it:C,func:F)->TreeAllocDstDfsOrder<T>{


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

        
        struct Counter<'a>{
            counter:&'a 
            mut u8
        }
        impl<'a> Counter<'a>{
            fn add_node<T:SweepTrait,B,F:Fn(B,&mut NodeDyn<T>)>(&mut self,stuff:(usize,B),func:&F)->&'a mut NodeDstDyn<T>{
                let dst:&mut NodeDstDyn<T>=unsafe{std::mem::transmute(ReprMut{ptr:self.counter,size:stuff.0})};    
                
                dst.c=None; //We set the children later
                func(stuff.1,&mut dst.n);
                self.counter=unsafe{&mut *(self.counter as *mut u8).offset(std::mem::size_of_val(dst) as isize)};
                dst
            }
        }

        let mut cc=Counter{counter:start_addr};
        let root=recc(it,&func,&mut cc);
        
        return TreeAllocDstDfsOrder{_vec:vec,root};


        fn recc<'a,T:SweepTrait,B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<T>)>
            (it:C,func:&F,counter:&mut Counter<'a>)->&'a mut NodeDstDyn<T>{
            
            let (nn,rest)=it.next();
            
            return match rest{
                Some((left,right))=>{
                    let left=recc(left,func,counter);
                    
                    let mut node=counter.add_node(nn,func);
                    
                    let right=recc(right,func,counter);
                    
                    node.c=Some((left,right));
                    node
                    //Do stuff here! Now both children okay
                },
                None=>{
                    let mut node=counter.add_node(nn,func);
                    node.c=None;
                    node
                }
            };   
        }
    }
}

/*
//TODO should technically call the destructor of T.
pub struct TreeAllocDst<T:SweepTrait>{
    _vec:Vec<u8>,
    root:*mut NodeDstDyn<T>
}

impl<T:SweepTrait> TreeAllocDst<T>{   

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
            //TODO safe to do this??????????????
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