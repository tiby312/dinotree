use inner_prelude::*;

///A KdTree construction
///This is like DynTree except the size of every node is constant.
pub struct KdTree<'a,A:AxisTrait,T:HasAabb+'a> {
    tree: compt::dfs_order::GenTreeDfsOrder<Node2<'a,T>>,
    _p:PhantomData<A>
}

impl<'a,A:AxisTrait,T:HasAabb+Send+'a> KdTree<'a,A,T>{

    pub fn new<JJ:par::Joiner,K:TreeTimerTrait>(axis:A,rest:&'a mut [T],height:usize) -> (KdTree<'a,A,T>,K::Bag) {
        
        let mut ttree=compt::dfs_order::GenTreeDfsOrder::from_dfs_inorder(&mut ||{
            let rest=&mut [];
            //Get rid of zero initialization???
            let div=unsafe{std::mem::uninitialized()};
            Node2{div,range:rest}
            
        },height);

        let bag={
            let j=ttree.create_down_mut().with_depth(Depth(0));
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

    pub fn get_tree(&self)->&compt::dfs_order::GenTreeDfsOrder<Node2<'a,T>>{
        &self.tree
    }
    pub fn get_tree_mut(&mut self)->&mut compt::dfs_order::GenTreeDfsOrder<Node2<'a,T>>{
        &mut self.tree
    }
}

pub struct Node2<'a,T:HasAabb+'a>{ 

    //If this is a non leaf node, then,
    //  div is None iff this node and children nodes do not have any bots in them.
    // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
    // This is because we specifically pick the median.
    // If it is a leaf node, then div being none still means it could have bots in it.
    //pub div:Option<(T::Num,axgeom::Range<T::Num>)>,
    pub div:tree_alloc::FullComp<T::Num>,
    pub range:&'a mut [T]
}


fn recurse_rebal<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:TreeTimerTrait>(
    div_axis:A,
    dlevel:JJ,
    rest:&'b mut [T],
    down:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>,
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
            //nn.div=None;

            nn.range=rest;
            timer_log.leaf_finish()
        },
        Some(((),lleft,rright))=>{
            let lleft:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>=lleft;
            let rright:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>=rright;
            
            let med = if rest.len() == 0{
                return timer_log.leaf_finish();
            }
            else
            {

                let closure = |a: &T, b: &T| -> std::cmp::Ordering {
                    oned::compare_bots(div_axis,a,b)
                };

                let k={
                    let mm=rest.len()/2;
                    pdqselect::select_by(rest, mm, closure);
                    &rest[mm]
                };

                k.get().get_range(div_axis).left
            };

            //It is very important that the median bot end up be binned into the middile bin.
            //We know this must be true because we chose the divider to be the medians left border,
            //and we binned so that all bots who intersect with the divider end up in the middle bin.
            //Very important that if a bots border is exactly on the divider, it is put in the middle.
            //If this were not true, there is no guarentee that the middile bin has bots in it even
            //though we did pick a divider.
            let binned=oned::bin_middle_left_right(div_axis,&med,rest);
            
            debug_assert!(binned.middle.len()!=0);
            
            
                    
    
            let oned::Binned{left,middle,right}=binned;
            

            let binned_left=left;
            let binned_middle=middle;
            let binned_right=right;                
            
            let (ta,tb)=timer_log.next();

            let (nj,ba,bb)=if !dlevel.should_switch_to_sequential(level){
                let ((nj,ba),bb)={
                    let af=move || {
                        //We already know that the middile is non zero in length.
                        let container_box=unsafe{create_cont_non_zero_unchecked(div_axis,binned_middle)};
                        
                        sweeper_update(div_axis.next(),binned_middle);
                        let n:Node2<'b,_>=Node2{div:tree_alloc::FullComp{div:med,cont:container_box},range:binned_middle};
                    
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
                //We already know that the middile is non zero in length.
                let container_box=unsafe{create_cont_non_zero_unchecked(div_axis,binned_middle)};
                
                sweeper_update(div_axis.next(),binned_middle);
                let nj=Node2{div:tree_alloc::FullComp{div:med,cont:container_box},range:binned_middle};
                let ba=self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_left,lleft,ta);
                let bb=self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_right,rright,tb);
                (nj,ba,bb)
            };
            *nn=nj;
            K::combine(ba,bb)
        }
    }
}


///The slice that is passed MUST NOT BE ZERO LENGTH!!!
pub unsafe fn create_cont_non_zero_unchecked<A:AxisTrait,T:HasAabb>(axis:A,middle:&[T])->axgeom::Range<T::Num>{
  
     let left=match middle.iter().min_by(|a,b|{
        a.get().get_range(axis).left.cmp(&b.get().get_range(axis).left)
     }){
        Some(x)=>x,
        None=>std::hint::unreachable_unchecked()
     };

     let right=match middle.iter().max_by(|a,b|{
        a.get().get_range(axis).right.cmp(&b.get().get_range(axis).right)
     }){
        Some(x)=>x,
        None=>std::hint::unreachable_unchecked()
     };

     axgeom::Range{left:left.get().get_range(axis).left,right:right.get().get_range(axis).right}    
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
    
}