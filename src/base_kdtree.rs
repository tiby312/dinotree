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

    pub fn new<JJ:par::Joiner,K:TreeTimerTrait>(rest:&'a mut [T],height:usize) -> (KdTree<'a,A,T>,K::Bag) {
        
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
            self::recurse_rebal::<A,T,JJ,K>(rest,j,t)
        };
        (KdTree{tree:ttree,_p:PhantomData},bag)
    }

    pub fn get_tree(&self)->&compt::GenTree<Node2<'a,T>>{
        &self.tree
    }
    pub fn get_tree_mut(&mut self)->&mut compt::GenTree<Node2<'a,T>>{
        &mut self.tree
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


fn recurse_rebal<'b,A:AxisTrait,T:RebalTrait,JJ:par::Joiner,K:TreeTimerTrait>(
    rest:&'b mut [T],
    down:compt::LevelIter<compt::DownTMut<Node2<'b,T>>>,
    mut timer_log:K)->K::Bag{

    timer_log.start();
    
    let ((level,nn),restt)=down.next();

    match restt{
        None=>{
            sweeper_update::<_,A::Next>(rest);
            let container_box=self::create_container_rect::<A,_>(rest);
            let divider=std::default::Default::default();
            *nn=Node2{divider,container_box,range:rest};
            timer_log.leaf_finish()
        },
        Some((lleft,rright))=>{

            
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
                m
                
            };

            let binned=oned::bin::<A,_>(&med,rest);



            let binned_left=binned.left;
            let binned_middile=binned.middile;
            let binned_right=binned.right;                

            let (ta,tb)=timer_log.next();

            let (nj,ba,bb)=if !JJ::new().should_switch_to_sequential(level){
                
                let ((nj,ba),bb)={
                    let af=move || {
                        sweeper_update::<_,A::Next>(binned_middile);
                        let container_box=self::create_container_rect::<A,_>(binned_middile);
                        let nj=Node2{divider:med,container_box,range:binned_middile};
                        let ba=self::recurse_rebal::<A::Next,T,par::Parallel,K>(binned_left,lleft,ta);
                        (nj,ba)
                    };

                    let bf=move || {
                        self::recurse_rebal::<A::Next,T,par::Parallel,K>(binned_right,rright,tb)
                    };
                    rayon::join(af,bf)
                }; 
                (nj,ba,bb)
            }else{

                sweeper_update::<_,A::Next>(binned_middile);
                let container_box=self::create_container_rect::<A,_>(binned_middile);
                let nj=Node2{divider:med,container_box,range:binned_middile};
                let ba=self::recurse_rebal::<A::Next,T,par::Sequential,K>(binned_left,lleft,ta);
                let bb=self::recurse_rebal::<A::Next,T,par::Sequential,K>(binned_right,rright,tb);
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
    }
    */

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