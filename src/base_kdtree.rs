use inner_prelude::*;

/*
#[derive(Copy,Clone,Debug)]
pub struct DivNode<Nu:Ord+Copy+std::fmt::Debug>{
    divider:Nu    
}
impl<Nu:Ord+Copy+std::fmt::Debug> DivNode<Nu>{
    pub fn divider(&self)->&Nu{
        &self.divider
    }
}*/



///A KdTree construction
///This is like DynTree except the size of every node is constant.
pub struct KdTree<'a,A:AxisTrait,T:HasAabb+'a> {
    tree: compt::dfs::GenTreeDfsOrder<Node2<'a,T>>,
    _p:PhantomData<A>
}

impl<'a,A:AxisTrait,T:HasAabb+Send+'a> KdTree<'a,A,T>{

    pub fn new<JJ:par::Joiner,K:TreeTimerTrait>(axis:A,rest:&'a mut [T],height:usize) -> (KdTree<'a,A,T>,K::Bag) {
        
        let mut ttree=compt::dfs::GenTreeDfsOrder::from_dfs_inorder(&mut ||{
            let rest=&mut [];
            //let co=self::rect_make::create_container_rect::<A,_>(rest);
            Node2{div:None,cont:None,range:rest}
            
        },height);

        let bag={
            //let level=ttree.get_level_desc();
            let j=ttree.create_down_mut().with_depth(Depth(0));
            //let j=compt::LevelIter::new(ttree.create_down_mut(),level);
            let t=K::new(height);

            //on xps13 5 seems good
            const DEPTH_SEQ:usize=6;

            let gg=if height<=DEPTH_SEQ{
                0
            }else{
                height-DEPTH_SEQ
            };
            
            let dlevel=JJ::new(Depth(gg));
            self::recurse_rebal::<A,T,JJ,K>(axis,dlevel,rest,j,t)
        };


        (KdTree{tree:ttree,_p:PhantomData},bag)
    }

}

impl<'a,A:AxisTrait,T:HasAabb+'a> KdTree<'a,A,T>{

    pub fn get_tree(&self)->&compt::dfs::GenTreeDfsOrder<Node2<'a,T>>{
        &self.tree
    }
    pub fn get_tree_mut(&mut self)->&mut compt::dfs::GenTreeDfsOrder<Node2<'a,T>>{
        &mut self.tree
    }
    /*
    pub fn into_tree(self)->compt::dfs::GenTreeDfsOrder<Node2<'a,T>>{
        let KdTree{tree,_p}=self;
        tree
    }*/
}

pub struct Node2<'a,T:HasAabb+'a>{ 

    //If this is a non leaf node, then,
    //  div is None iff this node and children nodes do not have any bots in them.
    //If it is a leaf node, then div being none still means it could have bots in it.
    pub div:Option<T::Num>,
 
    //box is None iff range.len()==0
    pub cont:Option<axgeom::Range<T::Num>>,
    
    pub range:&'a mut [T]
}



fn recurse_rebal<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:TreeTimerTrait>(
    div_axis:A,
    dlevel:JJ,
    rest:&'b mut [T],
    down:compt::LevelIter<compt::dfs::DownTMut<Node2<'b,T>>>,
    mut timer_log:K)->K::Bag{

    timer_log.start();
    
    let ((level,nn),restt)=down.next();

    match restt{
        None=>{
            //We are guarenteed that the leaf nodes have at most 10 bots
            //since we divide based off of the median, and picked the height
            //such that the leaves would have at most 10.
            oned::sweeper_update_leaf(div_axis.next(),rest);
            
            //Unsafely leave the dividers of leaf nodes uninitialized.
            //nn.divider=std::default::Default::default();
            //nn.container_box=rect_make::create_container_rect::<A,_>(rest);
         
            nn.range=rest;
            timer_log.leaf_finish()
        },
        Some((lleft,rright))=>{
            let lleft:compt::LevelIter<compt::dfs::DownTMut<Node2<'b,T>>>=lleft;
            let rright:compt::LevelIter<compt::dfs::DownTMut<Node2<'b,T>>>=rright;
            
            let med={
              
                let m = if rest.len() == 0{
                        None
                        //panic!("no bots in a non leaf node! depth:{:?}",level.0);
                        //std::default::Default::default()
                    }
                    else
                    {
                        let closure = |a: &T, b: &T| -> std::cmp::Ordering {
        
                            let arr=a.get().as_axis().get(div_axis);
                            let brr=b.get().as_axis().get(div_axis);
                      
                            if arr.left > brr.left{
                                return std::cmp::Ordering::Greater;
                            
                            }
                            std::cmp::Ordering::Less
                        };

                        let k={
                            let mm=rest.len()/2;
                            pdqselect::select_by(rest, mm, closure);
                            &rest[mm]
                        };
                        //Some(k)
                        Some(k.get().as_axis().get(div_axis).left)
                    };
                m
                
            };

            let med=match med{
                Some(med)=>{
                    med
                },
                None=>{
                    //TODO turn into debug assert
                    assert_eq!(rest.len(),0);
                    return timer_log.leaf_finish();
                }
            };
            
            //It is very important that the median bot end up be binned into the middile bin.
            //We know this must be true because we chose the divider to be the medians left border,
            //and we binned so that all bots who intersect with the divider end up in the middle bin.
            //Very important that if a bots border is exactly on the divider, it is put in the middle.
            //If this were not true, there is no guarentee that the middile bin has bots in it even
            //though we did pick a divider.
            let binned=oned::bin_middle_left_right(div_axis,&med,rest);
            //TODO turn into debug assert
            assert!(binned.middle.len()!=0);
            
            
                    
    
            let oned::Binned{left,middle,right}=binned;
            

            let binned_left=left;
            let binned_middle=middle;
            let binned_right=right;                
            
            let (ta,tb)=timer_log.next();

            let (nj,ba,bb)=if !dlevel.should_switch_to_sequential(level){
                //println!("Parallel! {:?}",level.0);
                let ((nj,ba),bb)={
                    
                    let af=move || {

                        sweeper_update(div_axis.next(),binned_middle);
                        let container_box=rect_make::create_container_rect(div_axis,binned_middle);
                        let n:Node2<'b,_>=Node2{div:Some(med),cont:container_box,range:binned_middle};
                    
                        let k=self::recurse_rebal(div_axis.next(),dlevel,binned_left,lleft,ta);
                        (n,k)
                    };

                    let bf=move || {
                        self::recurse_rebal(div_axis.next(),dlevel,binned_right,rright,tb)
                    };
                    rayon::join(af,bf)
                }; 
                (nj,ba,bb)
            }else{
                //println!("Sequential {:?}",level.0);
                sweeper_update(div_axis.next(),binned_middle);
                let container_box=rect_make::create_container_rect(div_axis,binned_middle);
                let nj=Node2{div:Some(med),cont:container_box,range:binned_middle};
                let ba=self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_left,lleft,ta);
                let bb=self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_right,rright,tb);
                (nj,ba,bb)
            };

            *nn=nj;
            K::combine(ba,bb)
        }
    }
}

mod rect_make{
    use super::*;
    /*
    #[cfg(test)]
    mod test{
        use super::*;
        use test_support::*;
        use test_support;
        use support::*;
        use test::black_box;
        use test::Bencher;
        use oned::*;
        use axgeom;
        struct Bot{
            id:usize
        }

        #[bench]
        fn bench_rect_par(b:&mut Bencher){

            let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

            let mut bots=Vec::new();
            for id in 0..100000{
                let ppp=p.random_point();
                let k=test_support::create_rect_from_point(ppp);
                bots.push(BBox::new(Bot{id},k)); 
            }
            
            b.iter(||{
                black_box(create_container_rect_par::<axgeom::XAXIS_S,_>(&mut bots));
            });
            
        }

        #[bench]
        fn bench_rect(b:&mut Bencher){

            let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

            let mut bots=Vec::new();
            for id in 0..100000{
                let ppp=p.random_point();
                let k=test_support::create_rect_from_point(ppp);
                bots.push(BBox::new(Bot{id},k)); 
            }
            
            b.iter(||{
                black_box(create_container_rect::<axgeom::XAXIS_S,_>(&mut bots));
            });
            
        }


        pub fn create_container_rect_par<A:AxisTrait,T:HasAabb>(middile:&[T])->axgeom::Range<T::Num>{
            use rayon::prelude::*;

            {
                let res=middile.split_first();

                match res{
                    Some((first,rest))=>{

                        let first_ra=first.get().get_range2::<A>().clone();
                        
                        use smallvec::SmallVec;
                        let mut vecs:SmallVec<[&[T];16]> =rest.chunks(2000).collect();

                        let res:axgeom::Range<T::Num>=
                            vecs.par_iter().map(|a|{create_container::<A,T>(a,first_ra)}).
                            reduce(||first_ra,|a,b|merge(a,b));
                        res
                    },
                    None=>{
                        
                        let d=std::default::Default::default();
                        axgeom::Range{start:d,end:d}
                    }

                }
            }
        }

        fn merge<T:NumTrait>(mut a:axgeom::Range<T>,b:axgeom::Range<T>)->axgeom::Range<T>{
            a.grow_to_fit(&b);
            a
        }
    }
    */
    

    pub fn create_container_rect<A:AxisTrait,T:HasAabb>(axis:A,middile:&[T])->Option<axgeom::Range<T::Num>>{
        
        {
            let res=middile.split_first();

            match res{
                Some((first,rest))=>{

                    let first_ra=first.get().as_axis().get(axis).clone();
                    
                    Some(create_container(axis,rest,first_ra))
                },
                None=>{
                    None
                    //panic!("trying to create container rect of empty list!");
                    //let d=std::default::Default::default();
                    //axgeom::Range{start:d,end:d}
                }

            }
        }
    }
    fn create_container<A:AxisTrait,T:HasAabb>(axis:A,rest:&[T],mut container_rect:axgeom::Range<T::Num>)->axgeom::Range<T::Num>{
        
        for i in rest{
            container_rect.grow_to_fit(i.get().as_axis().get(axis));
        }
        container_rect
   
    }
}