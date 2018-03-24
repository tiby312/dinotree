
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
    tree:DynTreeRaw<T>,
    mover:Mover,
    _p:PhantomData<A>
}

#[cfg(test)]
mod test{
    use support::BBox;
    use super::*;
    use test::*;
    #[bench]
    fn method1(b:&mut Bencher){
         use test_support::*;
        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..50000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id,col:Vec::new()},k)); 
        }
        
        let height=compute_tree_height(bots.len());
        b.iter(||{
            black_box(DynTree::<XAXISS,_>::new::<par::Parallel,TreeTimerEmpty>(&mut bots,height));
        });
    }
    #[bench]
    fn method_exp(b:&mut Bencher){
         use test_support::*;
        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..50000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id,col:Vec::new()},k)); 
        }
        
        let height=compute_tree_height(bots.len());
        b.iter(||{
            black_box(DynTree::<XAXISS,_>::from_exp_method::<par::Parallel,TreeTimerEmpty>(&mut bots,height));
        });
    }
    #[bench]
    fn method_exp2(b:&mut Bencher){
         use test_support::*;
        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..50000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id,col:Vec::new()},k)); 
        }
        
        let height=compute_tree_height(bots.len());
        b.iter(||{
            black_box(DynTree::<XAXISS,_>::from_exp2_method::<par::Parallel,TreeTimerEmpty>(&mut bots,height));
        });
    }
}


impl<'a,A:AxisTrait,T:SweepTrait+'a> DynTree<'a,A,T>{


    fn method_exp<JJ:par::Joiner,K:TreeTimerTrait>(rest:&'a mut [T],height:usize)->(DynTreeRaw<T>,Mover,K::Bag){
        
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
                
                dst.divider=builder.divider;
                dst.container_box=builder.container_box;

                
                for (a,b) in dst.range.iter_mut().zip(builder.range.iter()){
                    //let k=&mut all_bots[b.index as usize];
                    //we cant just move it into here.
                    //then rust will try and call the destructor of the uninitialized object
                    let b=&rest[b.index as usize];
                   
                    unsafe{std::ptr::copy(b,a,1)};
                    std::mem::forget(b);
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

        (DynTree{orig:rest,tree:fb,mover,_p:PhantomData},bag)
    }

    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    pub fn get_level_desc(&self)->LevelDesc{
        self.tree.get_level()
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


use self::alloc::DynTreeRaw;
mod alloc{
    use super::*;
    //use tree_alloc::TreeAllocDst;
    //use tree_alloc::NodeDynBuilder; 
    use tree_alloc::TreeAllocDstDfsOrder;

    pub struct DynTreeRaw<T:SweepTrait>{
        height:usize,
        level:LevelDesc,
        alloc:TreeAllocDstDfsOrder<T>,
    
    }

    impl<T:SweepTrait+Send> DynTreeRaw<T>{
        pub fn new<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<T>)>(height:usize,level:LevelDesc,num_nodes:usize,num_bots:usize,ir:C,func:F)->DynTreeRaw<T>{
            let alloc=TreeAllocDstDfsOrder::new(num_nodes,num_bots,ir,func);
            DynTreeRaw{height,level,alloc}
        }
        pub fn get_level(&self)->LevelDesc{
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

        pub unsafe fn move_into_tree<T>(a:&T)->T{
            let mut k=std::mem::uninitialized();
            std::ptr::copy(a,&mut k,1);
            k
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
