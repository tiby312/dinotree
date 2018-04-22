
use inner_prelude::*;
use tree_alloc::NodeDyn;
use base_kdtree::Node2;
use base_kdtree::KdTree;
use base_kdtree::RebalTrait;
use tree_alloc::NdIterMut;
use tree_alloc::NdIter;
use compt::CTreeIterator;



pub struct DynTree<'b,A:AxisTrait,T:SweepTrait+Send+'b>{
    orig:&'b mut [T],
    mover:Mover,
    pub tree:DynTreeRaw<A,T>,
}



impl<'a,A:AxisTrait,T:SweepTrait+'a> DynTree<'a,A,T>{
    fn assert_invariants(&self){
        let c=self.get_iter().with_depth();


        fn recc<'a,A:AxisTrait,T:SweepTrait+'a,C:CTreeIterator<Item=(Depth,&'a NodeDyn<T>)>>(axis:A,cc:C){
            let ((depth,nn),rest)=cc.next();

                
            let div=match nn.div{
                Some(div)=>{div},
                None=>{return;}
            };

            for b in &nn.range{
                let r=(b.get().0).0.get_range(A::get());
                assert!(r.start<=div && r.end>=div);
            }        

            match rest{
                Some((left,right))=>{
                    recc(axis.next(),left);
                    recc(axis.next(),right);
                },
                None=>{

                }
            }
        }
        recc(A::new(),c);

    }

    fn method_exp<JJ:par::Joiner,K:TreeTimerTrait>(rest:&'a mut [T],height:usize)->(DynTreeRaw<A,T>,Mover,K::Bag){
        
        pub struct Cont2<N:NumTrait>{
            rect:Rect<N>,
            pub index:u32
        }
        impl<N:NumTrait> RebalTrait for Cont2<N>{
            type Num=N;
            fn get(& self)->&Rect<N>{
                &self.rect
            }
        }

        let num_bots=rest.len();
        let mut conts:Vec<Cont2<T::Num>>=rest.iter().enumerate().map(|(index,k)|{
            Cont2{rect:(k.get().0).0,index:index as u32}
        }).collect();
        
        {
            let (mut tree2,bag)=KdTree::<A,_>::new::<JJ,K>(&mut conts,height);
            
            
            let mover={

                let k=tree2.get_tree().dfs_inorder_iter().flat_map(|a:&Node2<Cont2<T::Num>>|{
                    a.range.iter()
                }).map(|a|a.index);

                Mover::new::<T::Num,_>(num_bots,k)
            };
            

            let height=tree2.get_tree().get_height();                
            let leveld=tree2.get_tree().get_level_desc();
            let num_nodes=tree2.get_tree().get_nodes().len();


            let ii=tree2.get_tree_mut().create_down_mut().map(|node:&mut Node2<Cont2<T::Num>>|{
                (node.range.len(),node as &Node2<Cont2<T::Num>>)
            });

            let func=|builder:&Node2<Cont2<T::Num>>,dst:&mut NodeDyn<T>|{
                
                dst.div=builder.div;
                dst.cont=builder.cont;
                //dst.divider=builder.divider;
                //dst.container_box=builder.container_box;

                
                for (a,b) in dst.range.iter_mut().zip(builder.range.iter()){
                    //let k=&mut all_bots[b.index as usize];
                    //we cant just move it into here.
                    //then rust will try and call the destructor of the uninitialized object
                    let bb=&rest[b.index as usize];
                   
                    unsafe{std::ptr::copy(bb,a,1)};
                    std::mem::forget(bb);
                }
                
            };
            let fb=DynTreeRaw::new(height,leveld,num_nodes,num_bots,ii,func);
            
            (fb,mover,bag)
        }
    }

    ///Create the tree.
    ///Specify whether it is done in parallel or sequential.
    ///If parallel, also specify the depth at which to switch to sequential.
    ///Also specify the median finding strategy to use.
    ///Also specify whether to use collect timing dat.a
    pub fn new<JJ:par::Joiner,K:TreeTimerTrait>(
        rest:&'a mut [T],height:usize) -> (DynTree<'a,A,T>,K::Bag) {


        assert!(rest.len()<u32::max_value() as usize,"Slice too large. The max slice size is {:?}",u32::max_value());

        //let num_bots=rest.len();
        let (fb,mover,bag)={
            //This one is the fastest when benching on phone and pc.
            Self::method_exp::<JJ,K>(rest,height)
        };

        let d=DynTree{orig:rest,mover,tree:fb};
        //TODO remove
        //d.assert_invariants();
        (d,bag)
    }

    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    pub fn get_level_desc(&self)->Depth{
        self.tree.get_level_desc()
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,T>{
        self.tree.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,T>{
        self.tree.get_iter()
    }
}



impl<'a,A:AxisTrait,T:SweepTrait+Send+'a> Drop for DynTree<'a,A,T>{
    fn drop(&mut self){
        let orig=&mut self.orig;


        unsafe{
            self.mover.move_out_of_tree(self.tree.get_iter(),orig);
            
        }
    }
}


pub use self::alloc::DynTreeRaw;

mod alloc{
    use super::*;
    //use tree_alloc::TreeAllocDst;
    //use tree_alloc::NodeDynBuilder; 
    use tree_alloc::TreeAllocDstDfsOrder;

    pub struct DynTreeRaw<A:AxisTrait,T:SweepTrait>{
        height:usize,
        level:Depth,
        alloc:TreeAllocDstDfsOrder<T>,
        _p:PhantomData<A>
    }

    impl<A:AxisTrait,T:SweepTrait+Send> DynTreeRaw<A,T>{
        pub fn new<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<T>)>(height:usize,level:Depth,num_nodes:usize,num_bots:usize,ir:C,func:F)->DynTreeRaw<A,T>{
            let alloc=TreeAllocDstDfsOrder::new(num_nodes,num_bots,ir,func);
            DynTreeRaw{height,level,alloc,_p:PhantomData}
        }
        pub fn get_level_desc(&self)->Depth{
            self.level
        }
        pub fn get_height(&self)->usize{
            self.height
        }

        pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,T>{
            self.alloc.get_iter_mut()
        }
        pub fn get_iter<'b>(&'b self)->NdIter<'b,T>{
            self.alloc.get_iter()
        }
    }
}


use self::mover::Mover;
mod mover{
    use std;
    use NumTrait;
    use compt::CTreeIterator;
    use tree_alloc::NodeDyn;
    use SweepTrait;
    //use dyntree::IdCont;

    pub struct Mover(
        Vec<u32>
    );

    impl Mover{
        pub fn new<'b,T:NumTrait+'b,I:Iterator<Item=u32>>(num_bots:usize,iter:I)->Mover{
            let mut move_vector=Vec::with_capacity(num_bots);    
            
            for index in iter{
                move_vector.push(index);
            }

            Mover(move_vector)
        }

        pub unsafe fn move_out_of_tree<'a,T:'a+SweepTrait,C:CTreeIterator<Item=&'a NodeDyn<T>>>(&mut self,tree_bots:C,orig:&mut [T]){

            let mut i1=self.0.iter();
            tree_bots.dfs_inorder(|node|{
                for b in node.range.iter(){
                    let mov=i1.next().unwrap();
                    
                    let cp=&mut orig[*mov as usize];

                    std::ptr::copy(b,cp,1);
                    
                }
            });
        }
    }
}
