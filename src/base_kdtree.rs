use inner_prelude::*;

///A KdTree construction
///This is like DynTree except the size of every node is constant.
pub struct KdTree<'a,A:AxisTrait,T:HasAabb+'a> {
    tree: compt::dfs_order::GenTreeDfsOrder<Node2<'a,T>>,
    _p:PhantomData<A>
}

impl<'a,A:AxisTrait,T:HasAabb+Send+'a> KdTree<'a,A,T>{

    pub fn new<JJ:par::Joiner,K:Splitter+Send>(axis:A,rest:&'a mut [T],height:usize,splitter:K,par:JJ) -> (KdTree<'a,A,T>,K) {
        
        let mut ttree=compt::dfs_order::GenTreeDfsOrder::from_dfs_inorder(&mut ||{
            let rest=&mut [];
            //Get rid of zero initialization???
            let div=unsafe{std::mem::uninitialized()};
            Node2{div,range:rest}
            
        },height);

        let bag={
            let j=ttree.create_down_mut().with_depth(Depth(0));

            self::recurse_rebal::<A,T,JJ,K>(axis,par,rest,j,splitter)
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





fn recurse_rebal<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'b mut [T],
    down:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>,
    mut splitter:K)->K{
    splitter.node_start();

    let ((level,nn),restt)=down.next();

    match restt{
        None=>{
            //We are guarenteed that the leaf nodes have at most 10 bots
            //since we divide based off of the median, and picked the height
            //such that the leaves would have at most 10.
            oned::sweeper_update(div_axis.next(),rest);
            
            //Unsafely leave the dividers of leaf nodes uninitialized.
            //nn.divider=std::default::Default::default();
            //nn.container_box=rect_make::create_container_rect::<A,_>(rest);
            //nn.div=None;

            nn.range=rest;
            splitter.node_end();
            splitter //TODO is this okay?
        },
        Some(((),lleft,rright))=>{
            let lleft:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>=lleft;
            let rright:compt::LevelIter<compt::dfs_order::DownTMut<Node2<'b,T>>>=rright;
            
            let med = if rest.len() == 0{
                splitter.node_end();
                return splitter; //TODO is this okay?
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

            //We already know that the middile is non zero in length.
            let container_box=unsafe{create_cont_non_zero_unchecked(div_axis,binned_middle)};
            
            sweeper_update(div_axis.next(),binned_middle);
            let nj:Node2<'b,_>=Node2{div:tree_alloc::FullComp{div:med,cont:container_box},range:binned_middle};
            *nn=nj;

            let (splitter1,splitter2)=splitter.div();

            let (splitter1,splitter2)=if !dlevel.should_switch_to_sequential(level){
                let af=move || {self::recurse_rebal(div_axis.next(),dlevel,binned_left,lleft,splitter1)};
                let bf=move || {self::recurse_rebal(div_axis.next(),dlevel,binned_right,rright,splitter2)};
                rayon::join(af,bf)
            }else{
                (self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_left,lleft,splitter1),
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_right,rright,splitter2))
            };
            splitter1.add(splitter2)
            
        }
    }
}


///The slice that is passed MUST NOT BE ZERO LENGTH!!!
unsafe fn create_cont_non_zero_unchecked<A:AxisTrait,T:HasAabb>(axis:A,middle:&[T])->axgeom::Range<T::Num>{
  
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
