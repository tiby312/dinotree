use inner_prelude::*;
//use median::strict::*;

#[derive(Copy,Clone,Debug)]
pub struct DivNode<Nu:Ord+Copy+std::fmt::Debug>{
    divider:Nu    
}
impl<Nu:Ord+Copy+std::fmt::Debug> DivNode<Nu>{
    pub fn divider(&self)->&Nu{
        &self.divider
    }
}

/*
///This preserves some state of the medians at each level between kdtree constructions.
pub struct TreeCache<A:AxisTrait,Nu:NumTrait>{
    height:usize,
    medtree:compt::GenTree<DivNode<Nu>>,
    _p:PhantomData<A>
}


impl<A:AxisTrait,Nu:NumTrait> TreeCache<A,Nu>{
    ///The tree cache contains within it a tree to keep a persistant state between construction of the kdtree.
    ///So the height of the kdtree is decided here, before the creation of the tree.
    pub fn new(height:usize)->TreeCache<A,Nu>{
        //let num_nodes=compt::compute_num_nodes(height);
        
        let t= compt::GenTree::from_bfs(&mut ||{DivNode{divider:std::default::Default::default()}},height);

        TreeCache{medtree:t,height:height,_p:PhantomData}
    }

    pub fn get_tree(&self)->&compt::GenTree<DivNode<Nu>>{
        &self.medtree
    }
}
*/
pub trait RebalTrait:Send+Sync{
    type Num:NumTrait;
    fn get(&self)->&axgeom::Rect<Self::Num>;
}


///A KdTree construction
pub struct KdTree<'a,A:AxisTrait,T:RebalTrait+'a> {
    tree: compt::GenTree<Node2<'a,T>>,
    _p:PhantomData<A>
}

impl<'a,A:AxisTrait,T:RebalTrait+'a> KdTree<'a,A,T>{

    pub fn new<JJ:par::Joiner,H:DepthLevel,K:TreeTimerTrait>(rest:&'a mut [T],height:usize) -> (KdTree<'a,A,T>,K::Bag) {
        
        //TODO replace with an unitialized version of thetree?
        //to reduce overhead???
        let mut ttree=compt::GenTree::from_bfs(&mut ||{
            let rest=&mut [];
            use std;

            let co=self::create_container_rect::<A,_>(rest);
            
            Node2{divider:std::default::Default::default(),container_box:co,range:rest}
        },height);

        let bag={

            let level=ttree.get_level_desc();
            //let m=tc.medtree.create_down_mut();
            let j=compt::LevelIter::new(ttree.create_down_mut(),level);
            let t=K::new(height);
            self::recurse_rebal::<A,T,H,JJ,K>(rest,j,t)
        };
        (KdTree{tree:ttree,_p:PhantomData},bag)
    }

    pub fn get_tree(&self)->&compt::GenTree<Node2<'a,T>>{
        &self.tree
    }

    pub fn into_tree(self)->compt::GenTree<Node2<'a,T>>{
        let KdTree{tree,_p}=self;
        tree
    }
}


pub struct Node2<'a,T:RebalTrait+'a>{ 

    pub divider:T::Num,

    //only valid if the node has bots in it.
    pub container_box:axgeom::Range<T::Num>,

    pub range:&'a mut [T]
}


fn recurse_rebal<'b,A:AxisTrait,T:RebalTrait,H:DepthLevel,JJ:par::Joiner,K:TreeTimerTrait>(
    rest:&'b mut [T],
    down:compt::LevelIter<compt::DownTMut<Node2<'b,T>>>,
    mut timer_log:K)->K::Bag{

    timer_log.start();
    
    let ((level,nn),restt)=down.next();

    /*
    let a = std::time::Duration::from_millis(300);
    if level.get_depth()==0{
        println!("sleepin!");
        std::thread::sleep(a);
    }
    */

    //let depth=level.get_depth();
    /*
    fn create_node<A:AxisTrait,T:SweepTrait,JJ:par::Joiner>(divider:T::Num,range:&mut [T])->Node2<T>{
        Sweeper::update::<A::Next,JJ>(range);
            
        let container_box=self::create_container_rect::<A,_>(range);
        Node2{divider,container_box,range}
    }
    */
    let mut tot_time=[0.0f64;4];
    match restt{
        None=>{
            sweeper_update::<_,A::Next,JJ>(rest);
            let container_box=self::create_container_rect::<A,_>(rest);
            let divider=std::default::Default::default();
            *nn=Node2{divider,container_box,range:rest};
            timer_log.leaf_finish()
        },
        Some((lleft,rright))=>{

            let tt0=tools::Timer2::new();
            //let (med,binned)=medianstrat.compute::<JJ,A,_>(level,rest,&mut div.divider);
            
            let med={
            
                
                let div_axis=A::get();
                let m = if rest.len() == 0{
                            std::default::Default::default()
                            //TODO what to do here?
                    }
                    else
                    {
                        let closure = |a: &T, b: &T| -> std::cmp::Ordering {
        
                            let arr=a.get().get_range(div_axis);
                            let brr=b.get().get_range(div_axis);
                      
                            if arr.left() > brr.left(){
                                return std::cmp::Ordering::Greater;
                            
                            }
                            std::cmp::Ordering::Less
                        };

                        let k={
                            let mm=rest.len()/2;
                            pdqselect::select_by(rest, mm, closure);
                            &rest[mm]
                        };
                        k.get().get_range(div_axis).start
                    };
                //*mmm=m;
                m
                
            };

            tot_time[0]=tt0.elapsed();

            /*
            let binned=if JJ::is_parallel(){
                oned::bin_par::<A,_>(&med,rest)
            }else{
                oned::bin::<A,_>(&med,rest)
            };
            */
            let tt0=tools::Timer2::new();

            let binned=oned::bin::<A,_>(&med,rest);


            tot_time[1]=tt0.elapsed();

            let binned_left=binned.left;
            let binned_middile=binned.middile;
            let binned_right=binned.right;                

            //let elapsed=timer.elapsed();
            //timer_log.add_to_depth(depth,elapsed);
            let (ta,tb)=timer_log.next();

            let (nj,ba,bb)=if JJ::is_parallel() && !H::switch_to_sequential(level){
                //let mut ll2=timer_log.clone_one_less_depth(); 
                
                let ((nj,ba),bb)={
                    let af=move || {
                        sweeper_update::<_,A::Next,JJ>(binned_middile);
                        let container_box=self::create_container_rect_par::<A,_>(binned_middile);
                        let nj=Node2{divider:med,container_box,range:binned_middile};
                        let ba=self::recurse_rebal::<A::Next,T,H,par::Parallel,K>(binned_left,lleft,ta);
                        (nj,ba)
                    };

                    let bf=move || {
                        self::recurse_rebal::<A::Next,T,H,par::Parallel,K>(binned_right,rright,tb)
                    };
                    rayon::join(af,bf)
                };
                //timer_log.combine_one_less(ll2);  
                (nj,ba,bb)
            }else{

                let tt0=tools::Timer2::new();

                sweeper_update::<_,A::Next,JJ>(binned_middile);
                tot_time[2]=tt0.elapsed();


                //let ll=binned_middile.len();
                let tt0=tools::Timer2::new();

                let container_box=self::create_container_rect::<A,_>(binned_middile);
                tot_time[3]=tt0.elapsed();

                if level.get_depth()==0{
                    println!("container box={:?}",container_box);
                    println!("time dist={:?}",tot_time);
                }
                let nj=Node2{divider:med,container_box,range:binned_middile};
                

                let ba=self::recurse_rebal::<A::Next,T,H,par::Sequential,K>(binned_left,lleft,ta);
                let bb=self::recurse_rebal::<A::Next,T,H,par::Sequential,K>(binned_right,rright,tb);
                (nj,ba,bb)
            };

                

            *nn=nj;
            K::combine(ba,bb)
        }
    }
}


use self::bla::create_container_rect;
use self::bla::create_container_rect_par;
mod bla{
    use super::*;

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
    }

    pub fn create_container_rect<A:AxisTrait,T:RebalTrait>(middile:&[T])->axgeom::Range<T::Num>{
        
        {
            let res=middile.split_first();

            match res{
                Some((first,rest))=>{

                    let first_ra=first.get().get_range2::<A>().clone();
                    
                    create_container::<A,T>(rest,first_ra)
                },
                None=>{
                    
                    let d=std::default::Default::default();
                    axgeom::Range{start:d,end:d}
                }

            }
        }
    }
    pub fn create_container_rect_par<A:AxisTrait,T:RebalTrait>(middile:&[T])->axgeom::Range<T::Num>{
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
    fn create_container<A:AxisTrait,T:RebalTrait>(rest:&[T],mut container_rect:axgeom::Range<T::Num>)->axgeom::Range<T::Num>{
        
        for i in rest{
            container_rect.grow_to_fit(i.get().get_range2::<A>());
        }
        container_rect
   
    }
}