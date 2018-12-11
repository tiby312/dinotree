

pub mod dinotree;


use inner_prelude::*;




/// Reference to a node returned by the Vistr tree iterator.
pub struct NodeRef<'a,N,T>{
    pub misc:&'a N,
    pub range:&'a [T]
}

/// Reference to a node returned by the VistrMut tree iterator.
pub struct NodeRefMut<'a,N,T>{
    pub misc:&'a mut N,
    pub range:&'a mut [T]
}



pub trait Sorter:Copy+Clone+Send+Sync{
    fn sort(&self,axis:impl AxisTrait,bots:&mut [impl HasAabb]);
}

#[derive(Copy,Clone)]
pub struct DefaultSorter;

impl Sorter for DefaultSorter{
    fn sort(&self,axis:impl AxisTrait,bots:&mut [impl HasAabb]){
        oned::sweeper_update(axis,bots);
    }
}

#[derive(Copy,Clone)]
pub struct NoSorter;

impl Sorter for NoSorter{
    fn sort(&self,_axis:impl AxisTrait,_bots:&mut [impl HasAabb]){}
}


///A struct that contains data that only non leaf nodes contain.
#[derive(Copy,Clone)]
pub struct FullComp<N:NumTrait>{
    ///The position of the splitting line for this node.
    pub div:N,
    ///The 1d bounding box for this node. All bots that intersect the splitting line are 
    ///within this bounding box.
    pub cont:axgeom::Range<N> ,

}

#[derive(Copy,Clone)]
pub enum FullCompOrEmpty<N:NumTrait>{
    NonEmpty(FullComp<N>),
    Empty()
}




fn nodes_left(depth:usize,height:usize)->usize{
    let levels=height-depth;
    2usize.rotate_left(levels as u32)-1
}


mod cont_tree{

    use super::*;


    pub struct Node2<'a,Num:NumTrait+'a>{ 

        //If this is a non leaf node, then,
        //  div is None iff this node and children nodes do not have any bots in them.
        // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
        // This is because we specifically pick the median.
        // If it is a leaf node, then div being none still means it could have bots in it.
        pub fullcomp:FullCompOrEmpty<Num>,
        pub mid:&'a mut [Cont2<Num>]
    }




    #[derive(Copy,Clone)]
    pub struct Cont2<N:NumTrait>{
        pub rect:Rect<N>,
        pub index:u32
    }
    unsafe impl<N:NumTrait> HasAabb for Cont2<N>{
        type Num=N;
        fn get(&self)->&Rect<N>{
            &self.rect
        }
    }   





    pub struct ContTree<'a,Num:NumTrait>{
        tree:compt::dfs_order::CompleteTree<Node2<'a,Num>>,
        conts:&'a mut [Cont2<Num>]
    }

    impl<'a,Num:NumTrait> ContTree<'a,Num>{
        pub fn get_tree_mut(&mut self)->&mut compt::dfs_order::CompleteTree<Node2<'a,Num>>{
            &mut self.tree
        }
        pub fn get_tree(&self)->&compt::dfs_order::CompleteTree<Node2<'a,Num>>{
            &self.tree
        }
        pub fn get_conts_mut(&mut self)->&mut [Cont2<Num>]{
            self.conts
        }
        pub fn get_conts(&self)->&[Cont2<Num>]{
            self.conts
        }

        pub fn new<A:AxisTrait,JJ:par::Joiner,K:Splitter+Send>(div_axis:A,dlevel:JJ,rest:&'a mut [Cont2<Num>],sorter:impl Sorter,splitter:&mut K,height:usize,binstrat:BinStrat)->ContTree<'a,Num>{
            let rest2=unsafe{&mut *(rest as *mut [_])};
            let mut nodes=Vec::with_capacity(tree::nodes_left(0,height));
                
            //let mut nodes=Vec::new(); //TODO use with_capacity().
            recurse_rebal(div_axis,dlevel,rest,&mut nodes,sorter,splitter,0,height,binstrat);

            let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();
            ContTree{tree,conts:rest2}
        }
    }

    fn recurse_rebal<'a,A:AxisTrait,Num:NumTrait,JJ:par::Joiner,K:Splitter+Send>(
        div_axis:A,
        dlevel:JJ,
        rest:&'a mut [Cont2<Num>],
        nodes:&mut Vec<Node2<'a,Num>>,
        sorter:impl Sorter,
        splitter:&mut K,
        depth:usize,
        height:usize,
        binstrat:BinStrat){
        splitter.node_start();

        if depth<height-1{
            

            let mut splitter2=splitter.div();


            let (node,left,right)=match construct_non_leaf(binstrat,sorter,div_axis,rest){
                Some((fullcomp,left,mid,right))=>{
                    
                    (Node2{fullcomp:FullCompOrEmpty::NonEmpty(fullcomp),mid},left,right)
                },
                None=>{
                    //We don't want to return here since we still want to populate the whole tree!
                    (Node2{fullcomp:FullCompOrEmpty::Empty(),mid:&mut []},&mut [] as &mut [_],&mut [] as &mut [_]) //TODO rust should make this easier
                }
            };
            
            let splitter=if !dlevel.should_switch_to_sequential(Depth(depth)){
                let splitter2=&mut splitter2;

                let af= move || {
                    self::recurse_rebal(div_axis.next(),dlevel,left,nodes,sorter,splitter,depth+1,height,binstrat);
                    (splitter,nodes)
                };

                let bf= move || {

                    let mut nodes2:Vec<Node2<'a,Num>>=Vec::with_capacity(nodes_left(depth,height));
                    nodes2.push(node);                
                    self::recurse_rebal(div_axis.next(),dlevel,right,&mut nodes2,sorter,splitter2,depth+1,height,binstrat);
                    nodes2
                };
                let ((splitter,nodes),mut nodes2)=rayon::join(af,bf);
                nodes.append(&mut nodes2);
                splitter
            }else{
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),left,nodes,sorter,splitter,depth+1,height,binstrat);
                nodes.push(node);
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),right,nodes,sorter,&mut splitter2,depth+1,height,binstrat);
                splitter
            };
            
            splitter.add(splitter2);
        }else{
            let mut node=Node2{fullcomp:FullCompOrEmpty::Empty(),mid:rest};
            construct_leaf(sorter,div_axis,&mut node.mid);
            nodes.push(node);
            splitter.node_end();
        }
    }
}






pub fn create_cont<A:AxisTrait,T:HasAabb>(axis:A,middile:&[T])->Option<axgeom::Range<T::Num>>{
  

    let (first,rest)=middile.split_first()?;
    let mut min=first.get().get_range(axis).left;
    let mut max=first.get().get_range(axis).right;

    for a in rest.iter(){
        let left=a.get().get_range(axis).left;
        let right=a.get().get_range(axis).right;

        if left<min{
            min=left;
        }

        if right>max{
            max=right;
        }
    }

    Some(axgeom::Range{left:min,right:max})

}



pub fn construct_leaf<T:HasAabb>(sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T]){ 
    sorter.sort(div_axis.next(),bots);
}


#[allow(dead_code)]
#[derive(Copy,Clone,Debug)]
pub enum BinStrat{
    LeftMidRight,
    LeftMidRightUnchecked,
    MidLeftRight,
    LeftRightMid
}

pub fn construct_non_leaf<T:HasAabb>(bin_strat:BinStrat,sorter:impl Sorter,div_axis:impl AxisTrait,bots:&mut [T])->Option<(FullComp<T::Num>,&mut [T],&mut [T],&mut [T])>{
    let med=if bots.len() == 0{
        return None;
    }
    else
    {
        let closure = |a: &T, b: &T| -> std::cmp::Ordering {
            oned::compare_bots(div_axis,a,b)
        };

        let k={
            let mm=bots.len()/2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).left
    };

    
    for a in bots.iter(){ //TODO remove
        let a=a.get().get_range(div_axis);
        debug_assert!(a.left<=a.right);
    }
    
    
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned=match bin_strat{
        BinStrat::LeftMidRight=>{
            oned::bin_left_middle_right(div_axis,&med,bots)
        },
        BinStrat::LeftMidRightUnchecked=>{
            unsafe{oned::bin_left_middle_right_unchecked(div_axis,&med,bots)}
        },
        BinStrat::MidLeftRight=>{
            oned::bin_middle_left_right(div_axis,&med,bots)
        },
        BinStrat::LeftRightMid=>{
            oned::bin_left_right_middle(div_axis,&med,bots)
        }
    };

    debug_assert!(binned.middle.len()!=0);
    
    //We already know that the middile is non zero in length.
    let container_box=create_cont(div_axis,binned.middle).unwrap();
    
    sorter.sort(div_axis.next(),binned.middle);
    let full=FullComp{div:med,cont:container_box};
    Some((full,binned.left,binned.middle,binned.right))
}



///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user 
///cannot modify the bounding box since it is hidden, with only read access.
#[derive(Copy,Clone)]
pub struct BBox<N:NumTrait,T>{
    rect:Rect<N>,
    pub inner:T
}

use std::fmt::Formatter;
use std::fmt::Debug;

impl<N:NumTrait+Debug,T:Debug> Debug for BBox<N,T>{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result{
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

impl<N:NumTrait,T> BBox<N,T>{
    ///Unsafe since the user create to boxes whose rectangles do not intersect,
    ///but whose contents point to a shared resource thus violating the contract of HasAabb.
    #[inline]
    pub unsafe fn new(rect:Rect<N>,inner:T)->BBox<N,T>{
        BBox{rect,inner}
    }

}

unsafe impl<N:NumTrait,T> HasAabb for BBox<N,T>{
    type Num=N;
    #[inline]
    fn get(&self)->&Rect<Self::Num>{
        &self.rect
    }
}


///Some alternate strategies for rebalancing.
///Used to empirically show that the default (the first one) is a good default.
#[derive(Debug)]
pub enum RebalStrat{
    First,
    Second,
    Third
}




