use inner_prelude::*;
use advanced::Splitter;
use tree_alloc::BinStrat;


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




pub struct Node2<'a,T:HasAabb+'a>{ 

    //If this is a non leaf node, then,
    //  div is None iff this node and children nodes do not have any bots in them.
    // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
    // This is because we specifically pick the median.
    // If it is a leaf node, then div being none still means it could have bots in it.
    //pub div:Option<(T::Num,axgeom::Range<T::Num>)>,
    pub fullcomp:tree_alloc::FullComp<T::Num>,
    pub mid:&'a mut [T]
}

pub fn recurse_rebal<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
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
            nn.mid=rest;
            tree_alloc::construct_leaf(sorter,div_axis,&mut nn.mid);
            
            splitter.node_end();
        },
        Some(((),lleft,rright))=>{
            let (fullcomp,left,mid,right)=match tree_alloc::construct_non_leaf(BinStrat::MidLeftRight,sorter,div_axis,rest){
                Some(pass)=>{
                    pass
                },
                None=>{
                    return;
                }
            };

            let nj:Node2<'b,_>=Node2{fullcomp,mid};
            *nn=nj;



            let mut splitter2=splitter.div();

            let splitter=if !dlevel.should_switch_to_sequential(level){
                let splitter2=&mut splitter2;
                let af= move || {self::recurse_rebal(div_axis.next(),dlevel,left,lleft,sorter,splitter);splitter};
                let bf= move || {self::recurse_rebal(div_axis.next(),dlevel,right,rright,sorter,splitter2);};
                rayon::join(af,bf).0
            }else{
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),left,lleft,sorter,splitter);
                self::recurse_rebal(div_axis.next(),dlevel.into_seq(),right,rright,sorter,&mut splitter2);
                splitter
            };
            
            splitter.add(splitter2);
            
        }
    }
}



pub fn recurse_rebal2<'b,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'b mut [T],
    down:compt::LevelIter<compt::bfs_order_slice::VistrMut<Node2<'b,T>>>,
    sorter:impl Sorter,
    splitter:&mut K){
    splitter.node_start();

    let ((level,nn),restt)=down.next();


    match restt{
        None=>{
            nn.mid=rest;
            tree_alloc::construct_leaf(sorter,div_axis,&mut nn.mid);
            
            splitter.node_end();
        },
        Some(((),lleft,rright))=>{
            let (fullcomp,left,mid,right)=match tree_alloc::construct_non_leaf(BinStrat::LeftRightMid,sorter,div_axis,rest){
                Some(pass)=>{
                    pass
                },
                None=>{
                    //lleft.dfs_inorder_iter().map(|((level,nn),_)|nn.mid=&mut []);
                    //rright.dfs_inorder_iter().map(|((level,nn),_)|nn.mid=&mut []);
                    return;
                }
            };

            let nj:Node2<'b,_>=Node2{fullcomp,mid};
            *nn=nj;



            let mut splitter2=splitter.div();

            let splitter=if !dlevel.should_switch_to_sequential(level){
                let splitter2=&mut splitter2;
                let af= move || {self::recurse_rebal2(div_axis.next(),dlevel,left,lleft,sorter,splitter);splitter};
                let bf= move || {self::recurse_rebal2(div_axis.next(),dlevel,right,rright,sorter,splitter2);};
                rayon::join(af,bf).0
            }else{
                self::recurse_rebal2(div_axis.next(),dlevel.into_seq(),left,lleft,sorter,splitter);
                self::recurse_rebal2(div_axis.next(),dlevel.into_seq(),right,rright,sorter,&mut splitter2);
                splitter
            };
            
            splitter.add(splitter2);
            
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
