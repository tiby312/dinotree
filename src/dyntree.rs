
use inner_prelude::*;
use tree_alloc::NodeDyn;
use base_kdtree::Node2;
use base_kdtree::KdTree;
use HasAabb;
use tree_alloc::NdIterMut;
use tree_alloc::NdIter;
use tree_alloc::NdIterMove;
use compt::CTreeIterator;
use axgeom::*;

use tree_alloc::FullComp;

pub struct DynTree<A:AxisTrait,N,T:HasAabb>{
    mover:Mover,
    pub tree:DynTreeRaw<A,N,T>,
}



impl<A:AxisTrait,N:Copy,T:HasAabb> DynTree<A,N,T>{

    
    pub fn new(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->DynTree<A,N,T>{
        Self::new_inner::<par::Parallel,treetimer::TreeTimerEmpty,_>(axis,n,iter).0
    }
    pub fn new_seq(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->DynTree<A,N,T>{
        Self::new_inner::<par::Sequential,treetimer::TreeTimerEmpty,_>(axis,n,iter).0
    }
    pub fn with_debug(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->(DynTree<A,N,T>,Vec<f64>){
        let (a,b)=Self::new_inner::<par::Parallel,treetimer::TreeTimer2,_>(axis,n,iter);
        (a,b.into_vec())
    }
    pub fn with_debug_seq(axis:A,n:N,iter:impl ExactSizeIterator<Item=T>)->(DynTree<A,N,T>,Vec<f64>){
        let (a,b)=Self::new_inner::<par::Sequential,treetimer::TreeTimer2,_>(axis,n,iter);
        (a,b.into_vec())
    } 
    
    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot_mut<'a>(&'a mut self)->impl Iterator<Item=&'a mut T>{
        self.get_iter_mut().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter_mut())
    }

    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot<'a>(&'a self)->impl Iterator<Item=&'a T>{
        self.get_iter().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter())
    }
    

    pub fn with_extra<N2:Copy>(mut self,n2:N2)->DynTree<A,N2,T>{
        let (mover,fb)={
            let axis=self.tree.get_axis();
            

            let height=self.get_height();
            let num_nodes=self.tree.get_num_nodes();
            let num_bots=self.tree.get_num_bots();

            let mover=self.mover.clone();
            let ii=self.into_iterr().map(|node,eextra|{
                let l=tree_alloc::LeafConstructor{misc:n2,it:node.1};

                let extra=match eextra{
                    Some(extra)=>{
                        Some(tree_alloc::ExtraConstructor{
                            comp:extra
                        })
                    },
                    None=>{
                        None
                    }
                };

                (l,extra)
            });
            
            let tree=TreeAllocDstDfsOrder::new(ii,num_nodes,num_bots);
            (mover,DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree})
        };

        DynTree{mover,tree:fb}
    }
    

    pub fn compute_tree_health(&self)->f64{
        
        fn recc<N,T:HasAabb>(a:LevelIter<NdIter<N,T>>,counter:&mut usize,height:usize){
            let ((depth,nn),next)=a.next();
            match next{
                Some((extra,left,right))=>{
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


    pub fn debug_assert_invariants(&self){
        let c=self.get_iter().with_depth(Depth(0));


        fn recc<'a,A:AxisTrait,N:'a,T:HasAabb+'a>(axis:A,cc:LevelIter<NdIter<N,T>>){
            let ((_depth,nn),rest)=cc.next();

            match rest{
                Some((extra,left,right))=>{

                    let &FullComp{div,cont}=match extra{
                        Some(g)=>g,
                        None=>unimplemented!("FINISH THIS")
                    };

                    for b in &nn.range{
                        let r=b.get().as_axis().get(axis);
                        assert!(r.left<=div && r.right>=div);
                    }        


                    recc(axis.next(),left);
                    recc(axis.next(),right);
                },
                None=>{

                }
            }
        }
        recc(self.tree.get_axis(),c);

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

    fn method_exp<JJ:par::Joiner,K:TreeTimerTrait,I:ExactSizeIterator<Item=T>>(axis:A,n:N,iter:I)->(DynTreeRaw<A,N,T>,Mover,K::Bag){


        let height=compute_tree_height_heuristic(iter.len());

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
            let (mut tree2,bag)=KdTree::new::<JJ,K>(axis,&mut conts,height);
            
            
            let mover={

                let kk:Vec<u32>=tree2.get_tree().create_down().dfs_preorder_iter().flat_map(|(node,extra)|{
                    node.range.iter()
                }).map(|a|a.index).collect();

                Mover(kk)
            };
            

            let height=tree2.get_tree().get_height();                
            let num_nodes=tree2.get_tree().get_nodes().len();


            let tree={
                let ii=tree2.get_tree_mut().create_down_mut().map(|node,eextra|{
                    //(node.range.len(),node as &Node2<Cont2<T::Num>>)
                    let l=tree_alloc::LeafConstructor{misc:n,it:node.range.iter_mut().map(|b|{
                        //let k=&mut all_bots[b.index as usize];
                        //we cant just move it into here.
                        //then rust will try and call the destructor of the uninitialized object
                        let bb=&rest[b.index as usize];
                        let mut bot=unsafe{std::mem::uninitialized()};
                        unsafe{std::ptr::copy(bb,&mut bot,1)};
                       
                        bot
                    })};

                    let extra=match eextra{
                        Some(())=>{
                            Some(tree_alloc::ExtraConstructor{
                                comp:Some(node.div)
                            })
                        },
                        None=>{
                            None
                        }
                    };

                    (l,extra)
                });

                TreeAllocDstDfsOrder::new(ii,num_nodes,num_bots)
            };
            std::mem::forget(rest);

            let fb=DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree};
            
            (fb,mover,bag)
        }
    }



    ///Create the tree.
    ///Specify whether it is done in parallel or sequential.
    ///If parallel, also specify the depth at which to switch to sequential.
    ///Also specify the median finding strategy to use.
    ///Also specify whether to use collect timing dat.a
    fn new_inner<JJ:par::Joiner,K:TreeTimerTrait,I:ExactSizeIterator<Item=T>>(
        axis:A,
        n:N,
        iter:I) -> (DynTree<A,N,T>,K::Bag) {
        
        assert!(iter.len()<u32::max_value() as usize,"Slice too large. The max slice size is {:?}",u32::max_value());

        //let num_bots=rest.len();
        let (fb,mover,bag)={
            //This one is the fastest when benching on phone and pc.
            Self::method_exp::<JJ,K,I>(axis,n,iter)
        };

        let d=DynTree{mover,tree:fb};
        //TODO remove
        //d.assert_invariants();
        (d,bag)
    }

    pub fn get_axis(&self)->A{
        self.tree.get_axis()
    }
    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }


    pub fn into_iterr(mut self)->NdIterMove<N,T>{
        self.tree.into_iterr()
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.tree.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.tree.get_iter()
    }

    pub fn into_iter_orig_order(mut self)->impl ExactSizeIterator<Item=T>{
    
        let mut ret:Vec<T>=(0..self.mover.0.len()).map(|_|{
            unsafe{std::mem::uninitialized()}
        }).collect();

        let mut i1=self.mover.0.iter();     
        for (node,_) in self.tree.into_iterr().dfs_preorder_iter(){
            for bot in node.1{
                let mov=i1.next().unwrap();
                let cp=&mut ret[*mov as usize];
                unsafe{
                    std::ptr::copy(&bot,cp,1);
                }
                std::mem::forget(bot);
            }
        }
        ret.into_iter()

    }
}

//TODO get rid of this layer. It doesnt add anything.
use tree_alloc::TreeAllocDstDfsOrder;

pub struct DynTreeRaw<A:AxisTrait,N,T:HasAabb>{
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    alloc:TreeAllocDstDfsOrder<N,T>,
    axis:A
}

impl<A:AxisTrait,N,T:HasAabb> DynTreeRaw<A,N,T>{
   

    pub fn get_axis(&self)->A{
        self.axis
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

    pub fn into_iterr(mut self)->NdIterMove<N,T>{
        self.alloc.into_iterr()
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.alloc.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.alloc.get_iter()
    }
}



#[derive(Clone)]
pub struct Mover(
    pub Vec<u32>
);
