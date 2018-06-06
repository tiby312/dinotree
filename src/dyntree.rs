
use inner_prelude::*;
use tree_alloc::NodeDyn;
use base_kdtree::Node2;
use base_kdtree::KdTree;
use HasAabb;
use tree_alloc::NdIterMut;
use tree_alloc::NdIter;
use compt::CTreeIterator;




pub struct DynTree<A:AxisTrait,N,T:HasAabb>{
    mover:Mover,
    pub tree:DynTreeRaw<A,N,T>,
}


///Returns the height of what is used internally to construct a dinotree.
fn compute_tree_height(num_bots: usize) -> usize {
    
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.
    const NUM_PER_NODE: usize = 12;  

    //8 is worse than 20 which is worse than 12 on android. sticking with 12
    if num_bots <= NUM_PER_NODE {
        return 1;
    } else {
        return (num_bots as f32 / NUM_PER_NODE as f32).log2().ceil() as usize;
    }
}

impl<A:AxisTrait,N:Copy,T:HasAabb> DynTree<A,N,T>{


    pub fn with_extra<N2:Copy>(self,n2:N2)->DynTree<A,N2,T>{

        let func=|builder:(N2,&NodeDyn<N,T>),dst:&mut NodeDyn<N2,T>|{
            let (misc,builder)=builder;

            dst.div=builder.div;
            dst.cont=builder.cont;
 
            for (a,b) in dst.range.iter_mut().zip(builder.range.iter()){
                //let k=&mut all_bots[b.index as usize];
                //we cant just move it into here.
                //then rust will try and call the destructor of the uninitialized object
                //let bb=&rest[b.index as usize];
               
                unsafe{std::ptr::copy(b,a,1)};
                
            }  

            dst.misc=misc;
        };


        let fb={
            let ii=self.get_iter().with_extra(|_,b|{(b,b)},n2).map(|node:(N2,&NodeDyn<N,T>)|{
                    (node.1.range.len(),node)
                });;


            let height=self.get_height();
            let num_nodes=self.tree.get_num_nodes();//compt::compute_num_nodes(height);
            let num_bots=self.tree.get_num_bots();//orig.len();
            DynTreeRaw::new(height,num_nodes,num_bots,ii,func)
        };

        //TODO inefficient!!!!!
        //TODO dont call drop!!
        DynTree{mover:Mover(self.mover.0.clone()),tree:fb}
    
    }
    

    pub fn compute_tree_health(&self)->f64{
        
        fn recc<N,T:HasAabb>(a:LevelIter<NdIter<N,T>>,counter:&mut usize,height:usize){
            let ((depth,nn),next)=a.next();
            match next{
                Some((left,right))=>{
                    *counter+=nn.range.len()*(height-1-depth.0);
                    recc(left,counter,height);
                    recc(right,counter,height);
                },
                None=>{
                }
            }
        }
        let height=self.get_height();
        let mut counter=0;
        recc(self.get_iter().with_depth(Depth(0)),&mut counter,height);

        unimplemented!();
        //return counter/total as f64;
    }


    pub fn assert_invariants(&self){
        let c=self.get_iter().with_depth(Depth(0));


        fn recc<'a,A:AxisTrait,N:'a,T:HasAabb+'a,C:CTreeIterator<Item=(Depth,&'a NodeDyn<N,T>)>>(axis:A,cc:C){
            let ((_depth,nn),rest)=cc.next();

                
            let div=match nn.div{
                Some(div)=>{div},
                None=>{return;}
            };

            for b in &nn.range{
                let r=b.get().get_range(A::get());
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

        /*
        fn assert_correctness(&self,tree:&KdTree,botman:&BotMan)->bool{
            for (level,axis) in kd_axis::AxisIter::with_axis(tree.tree.get_level_iter()) {
                if level.get_depth()!=tree.tree.get_height()-1{
                    for n in level.iter(){
                        let no=tree.tree.get_node(n);
                        let cont_box=&no.container_box;// no.get_divider_box(&botman.prop,axis);

                        let arr=&tree.collision_botids[no.container.get_range().as_int_range()];
                        for b in arr{
                            let bot=botman.cont.get_bot(*b);
                            let circle=&botman.as_circle(bot);
                            assert!(cont_box.contains_circle(circle),"{:?}\n{:?}\n{:?}\n{:?}",no,(level,axis),cont_box,circle);
                        }
                    }
                }
                
            }
             

            let arr=&tree.collision_botids[tree.no_fit.end.0..];
            let mut cols=0;
            for (i, el1) in arr.iter().enumerate() {
                for el2 in arr[i + 1..].iter() {
                    let bb=(*el1,*el2);
                    let bots = botman.cont.get_bbotpair(bb);

                    match bot::is_colliding(&botman.prop, bots) {
                        Some(_) => {
                            cols+=1;
                        }
                        None => {
                        }
                    }
                }
            }

            let mut cls=0;
            for k in self.binner_helps.iter(){
                cls+=k.cols_found.len();
            }

            let lookup=|a:(BotIndex, BotIndex)|{
                for k in self.binner_helps.iter(){
                    for j in k.cols_found.iter(){
                        let aa=( (j.inds.0).0 ,(j.inds.1).0);
                        let bb=((a.0).0,(a.1).0);
                        if aa.0==bb.0 &&aa.1==bb.1{
                            return true;
                        }
                        if aa.0==bb.1 && aa.1==bb.0{
                            return true;
                        }
                    }
                }
                false            
            };
            if cols!=cls{
                println!("Cols fail! num collision exp:{:?},  calculated:{:?}",cols,cls);

                for (i, el1) in arr.iter().enumerate() {
                    for el2 in arr[i + 1..].iter() {
                        let bb=(*el1,*el2);
                        let bots = botman.cont.get_bbotpair(bb);

                        match bot::is_colliding(&botman.prop, bots) {
                            Some(_) => {
                                if !lookup(bb){
                                    println!("Couldnt find {:?}",(bb,bots));

                                    println!("in node:{:?}",(lookup_in_tree(tree,bb.0),lookup_in_tree(tree,bb.1)));
                                    let n1=lookup_in_tree(tree,bb.0).unwrap();
                                    let n2=lookup_in_tree(tree,bb.1).unwrap();
                                    let no1=tree.tree.get_node(n1);
                                    let no2=tree.tree.get_node(n2);
                                    
                                    println!("INTERSECTS={:?}",no1.cont.border.intersects_rect(&no2.cont.border));

                                }
                            }
                            None => {
                            }
                        }
                    }
                }
                assert!(false);
            }
            
            fn lookup_in_tree(tree:&BaseTree,b:BotIndex)->Option<NodeIndex>{
                for level in tree.tree.get_level_iter(){
                    for nodeid in level.iter().rev() {
                        
                        let n = tree.tree.get_node(nodeid);
                    
                        let k=n.container.get_range().as_int_range();

                        let arr=&tree.collision_botids[k];
                        for i in arr{
                            if b.0==i.0{
                                return Some(nodeid);
                            }
                        }
                    }
                }
                return None
            }
            true
        }*/
        /*
        //Note this doesnt check all invariants.
        //e.g. doesnt check that every bot is in the tree only once.
        fn assert_invariant<T:SweepTrait>(d:&DinoTree2<T>){
            
            let level=d.0.get_level_desc();
            let ll=compt::LevelIter::new(d.0.get_iter(),level);
            use compt::CTreeIterator;
            for (level,node) in ll.dfs_preorder_iter(){
               
               //println!("level={:?}",level.get_depth());
               if level.get_depth()%2==0{
                  oned::is_sorted::<A::Next,_>(&node.range);


                  let kk=node.container_box;
                  for a in node.range.iter(){
                     let (p1,p2)=(
                          a.get().0.get().get_range2::<A>().left(),
                          a.get().0.get().get_range2::<A>().right());
                      assert!(kk.left()<=p1);
                      assert!(p2<=kk.right());
                  }
               }else{
                  oned::is_sorted::<A,_>(&node.range);
                  
                  let kk=node.container_box;
                  for a in node.range.iter(){
                     let (p1,p2)=(
                          a.get().0.get().get_range2::<A::Next>().left(),
                          a.get().0.get().get_range2::<A::Next>().right());
                      assert!(kk.left()<=p1);
                      assert!(p2<=kk.right());
                  }
               }
            }       
            
        }
        */
    }

    fn method_exp<JJ:par::Joiner,K:TreeTimerTrait,I:ExactSizeIterator<Item=T>>(n:N,iter:I)->(DynTreeRaw<A,N,T>,Mover,K::Bag){


        let height=compute_tree_height(iter.len());

        let rest:Vec<T>=iter.collect();

        pub struct Cont2<N:NumTrait>{
            rect:Rect<N>,
            pub index:u32
        }
        impl<N:NumTrait> HasAabb for Cont2<N>{
            type Num=N;
            fn get(& self)->&Rect<N>{
                &self.rect
            }
        }

        let num_bots=rest.len();

        let mut conts:Vec<Cont2<T::Num>>=rest.iter().enumerate().map(|(index,k)|{
            Cont2{rect:*k.get(),index:index as u32}
        }).collect();
        
        {
            let (mut tree2,bag)=KdTree::<A,_>::new::<JJ,K>(&mut conts,height);
            
            
            let mover={

                let kk:Vec<u32>=tree2.get_tree().create_down().dfs_preorder_iter().flat_map(|a:&Node2<Cont2<T::Num>>|{
                    a.range.iter()
                }).map(|a|a.index).collect();

                Mover(kk)
            };
            

            let height=tree2.get_tree().get_height();                
            let num_nodes=tree2.get_tree().get_nodes().len();


            let ii=tree2.get_tree_mut().create_down_mut().map(|node:&mut Node2<Cont2<T::Num>>|{
                (node.range.len(),node as &Node2<Cont2<T::Num>>)
            });

            let func=|builder:&Node2<Cont2<T::Num>>,dst:&mut NodeDyn<N,T>|{
                
                dst.misc=n;
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
            let fb=DynTreeRaw::new(height,num_nodes,num_bots,ii,func);
            
            (fb,mover,bag)
        }
    }


    pub fn new(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->DynTree<A,N,T>{
        Self::new_inner::<par::Parallel,treetimer::TreeTimerEmpty,_>(n,iter).0
    }
    pub fn new_seq(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->DynTree<A,N,T>{
        Self::new_inner::<par::Sequential,treetimer::TreeTimerEmpty,_>(n,iter).0
    }
    pub fn with_debug(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->(DynTree<A,N,T>,Vec<f64>){
        let (a,b)=Self::new_inner::<par::Parallel,treetimer::TreeTimer2,_>(n,iter);
        (a,b.into_vec())
    }
    pub fn with_debug_seq(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->(DynTree<A,N,T>,Vec<f64>){
        let (a,b)=Self::new_inner::<par::Parallel,treetimer::TreeTimer2,_>(n,iter);
        (a,b.into_vec())
    }    
    ///Create the tree.
    ///Specify whether it is done in parallel or sequential.
    ///If parallel, also specify the depth at which to switch to sequential.
    ///Also specify the median finding strategy to use.
    ///Also specify whether to use collect timing dat.a
    fn new_inner<JJ:par::Joiner,K:TreeTimerTrait,I:ExactSizeIterator<Item=T>>(
        n:N,
        iter:I) -> (DynTree<A,N,T>,K::Bag) {
        
        assert!(iter.len()<u32::max_value() as usize,"Slice too large. The max slice size is {:?}",u32::max_value());

        //let num_bots=rest.len();
        let (fb,mover,bag)={
            //This one is the fastest when benching on phone and pc.
            Self::method_exp::<JJ,K,I>(n,iter)
        };

        let d=DynTree{mover,tree:fb};
        //TODO remove
        //d.assert_invariants();
        (d,bag)
    }

    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.tree.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.tree.get_iter()
    }

    pub fn into_iter(mut self)->impl ExactSizeIterator<Item=T>{
        
        
        let mut ret:Vec<T>=(0..self.mover.0.len()).map(|_|{
            unsafe{std::mem::uninitialized()}
        }).collect();
    
        let mut i1=self.mover.0.iter();
        self.tree.get_iter_mut().dfs_preorder_iter().map(|node|{
            for b in node.range.iter(){
                let mov=i1.next().unwrap();
                
                let cp=&mut ret[*mov as usize];
                unsafe{
                    std::ptr::copy(b,cp,1);
                }
            }
        });

        ret.into_iter()

    }
}



impl<A:AxisTrait,N,T:HasAabb> Drop for DynTree<A,N,T>{
    fn drop(&mut self){
        //TODO drop eveyrthing in the tree if it hasnt been moved out.
        unimplemented!();
    }
}


pub use self::alloc::DynTreeRaw;

mod alloc{
    use super::*;
    use tree_alloc::TreeAllocDstDfsOrder;

    pub struct DynTreeRaw<A:AxisTrait,N,T:HasAabb>{
        height:usize,
        num_nodes:usize,
        num_bots:usize,
        alloc:TreeAllocDstDfsOrder<N,T>,
        _p:PhantomData<A>
    }

    impl<A:AxisTrait,N,T:HasAabb> DynTreeRaw<A,N,T>{
        pub fn new<B,C:CTreeIterator<Item=(usize,B)>,F:Fn(B,&mut NodeDyn<N,T>)>(height:usize,num_nodes:usize,num_bots:usize,ir:C,func:F)->DynTreeRaw<A,N,T>{
            let alloc=TreeAllocDstDfsOrder::new(num_nodes,num_bots,ir,func);
            DynTreeRaw{height,num_nodes,num_bots,alloc,_p:PhantomData}
        }
        pub fn get_num_nodes(&self)->usize{
            self.num_nodes
        }
        pub fn get_num_bots(&self)->usize{
            self.num_bots
        }
        pub fn get_height(&self)->usize{
            self.height
        }

        pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
            self.alloc.get_iter_mut()
        }
        pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
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

    use HasAabb;
    //use HasAabb;
    //use dyntree::IdCont;

    pub struct Mover(
        pub Vec<u32>
    );

    impl Mover{

        pub unsafe fn move_out_of_tree<'a,N:'a,T:'a+HasAabb,C:CTreeIterator<Item=&'a NodeDyn<N,T>>>(&mut self,tree_bots:C,orig:&mut [T]){

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
