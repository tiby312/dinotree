use inner_prelude::*;
use advanced::Splitter;

///A KdTree construction
///This is like DynTree except the size of every node is constant.
pub struct DinoTreeInner<'a,A:AxisTrait,T:HasAabb+'a> {
    pub tree: compt::dfs_order::CompleteTree<Node2<'a,T>>,
    pub axis:A
}

impl<'a,A:AxisTrait,T:HasAabb+Send+'a> DinoTreeInner<'a,A,T>{

    pub fn new<JJ:par::Joiner,K:Splitter+Send>(axis:A,rest:&'a mut [T],height:usize,splitter:&mut K,par:JJ,sorter:impl Sorter) -> DinoTreeInner<'a,A,T> {
        
        let mut ttree=compt::dfs_order::CompleteTree::from_dfs_inorder(&mut ||{
            let rest=&mut [];
            //Get rid of zero initialization???
            let div=unsafe{std::mem::uninitialized()};
            Node2{div,range:rest}
            
        },height);

        {
            let j=ttree.vistr_mut().with_depth(Depth(0));

            self::recurse_rebal(axis,par,rest,j,sorter,splitter);
        }
        
        DinoTreeInner{tree:ttree,axis}
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
    fn sort(&self,axis:impl AxisTrait,bots:&mut [impl HasAabb]){}
}




fn recurse_rebal<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'b mut [T],
    down:compt::LevelIter<compt::dfs_order::VistrMut<Node2<'b,T>>>,
    sorter:impl Sorter,
    splitter:&mut K){
    splitter.node_start();

    let ((level,nn),restt)=down.next();

    match restt{
        None=>{
            sorter.sort(div_axis.next(),rest);
            //oned::sweeper_update(div_axis.next(),rest);
            
            nn.range=rest;
            //nn.div=std::default::Default::default();
            
            splitter.node_end();
        },
        Some(((),lleft,rright))=>{
            //let lleft:compt::LevelIter<compt::dfs_order::VistrMut<Node2<'b,T>>>=lleft;
            //let rright:compt::LevelIter<compt::dfs_order::VistrMut<Node2<'b,T>>>=rright;
            
            let med = if rest.len() == 0{
                /*
                //Initialize the nodes under this one
                for ((_,nn),_) in lleft.dfs_inorder_iter().chain(rright.dfs_inorder_iter()){
                    nn.range=&mut [];
                    nn.div=std::default::Default::default();
                }
                */
                
                splitter.node_end();
                return;
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
            let container_box=create_cont(div_axis,binned_middle).unwrap();
            
            //oned::sweeper_update(div_axis.next(),binned_middle);
            sorter.sort(div_axis.next(),binned_middle);
            let nj:Node2<'b,_>=Node2{div:tree_alloc::FullComp{div:med,cont:container_box},range:binned_middle};
            *nn=nj;



            let mut splitter2=splitter.div();

            let splitter=if !dlevel.should_switch_to_sequential(level){
                let splitter2=&mut splitter2;
                let af= move || {self::recurse_rebal(div_axis.next(),dlevel,binned_left,lleft,sorter,splitter);splitter};
                let bf= move || {self::recurse_rebal(div_axis.next(),dlevel,binned_right,rright,sorter,splitter2);};
                rayon::join(af,bf).0
            }else{
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_left,lleft,sorter,splitter);
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),binned_right,rright,sorter,&mut splitter2);
                splitter
            };
            
            splitter.add(splitter2);
            
        }
    }
}


fn create_cont<A:AxisTrait,T:HasAabb>(axis:A,middile:&[T])->Option<axgeom::Range<T::Num>>{
  

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
