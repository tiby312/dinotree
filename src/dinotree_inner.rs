use inner_prelude::*;


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
    pub fullcomp:FullCompOrEmpty<T::Num>,
    pub mid:&'a mut [T]
}


pub fn recurse_rebal1<'a,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'a mut [T],
    nodes:&mut Vec<Node2<'a,T>>,
    sorter:impl Sorter,
    splitter:&mut K,
    depth:usize,
    height:usize){
    splitter.node_start();



    if depth<height-1{
        

        let mut splitter2=splitter.div();


        let (node,left,right)=match construct_non_leaf(BinStrat::MidLeftRight,sorter,div_axis,rest){
            Some((fullcomp,left,mid,right))=>{
                
                (Node2{fullcomp:FullCompOrEmpty::NonEmpty(fullcomp),mid},left,right)
            },
            None=>{
                //We don't want to return here since we still want to populate the whole tree!
                (Node2{fullcomp:FullCompOrEmpty::Empty(),mid:&mut []},&mut [] as &mut [T],&mut [] as &mut [T]) //TODO rust should make this easier
            }
        };

        let splitter=if !dlevel.should_switch_to_sequential(Depth(depth)){
            let splitter2=&mut splitter2;

            let af= move || {
                self::recurse_rebal1(div_axis.next(),dlevel,left,nodes,sorter,splitter,depth+1,height);
                (splitter,nodes)
            };
            let bf= move || {
                let mut nodes2=Vec::new();
                nodes2.push(node);
                self::recurse_rebal1(div_axis.next(),dlevel,right,&mut nodes2,sorter,splitter2,depth+1,height);
                nodes2
            };
            let ((splitter,nodes),mut nodes2)=rayon::join(af,bf);
            nodes.append(&mut nodes2);
            splitter
        }else{
            self::recurse_rebal1(div_axis.next(),dlevel.into_seq(),left,nodes,sorter,splitter,depth+1,height);
            nodes.push(node);
            self::recurse_rebal1(div_axis.next(),dlevel.into_seq(),right,nodes,sorter,&mut splitter2,depth+1,height);
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



pub fn recurse_rebal2<'a,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'a mut [T],
    nodes:&mut SmallVec<[Node2<'a,T>;32]>,
    sorter:impl Sorter,
    splitter:&mut K,
    depth:usize,
    height:usize){
    splitter.node_start();



    if depth<height-1{
        

        let mut splitter2=splitter.div();


        let (node,left,right)=match construct_non_leaf(BinStrat::MidLeftRight,sorter,div_axis,rest){
            Some((fullcomp,left,mid,right))=>{
                
                (Node2{fullcomp:FullCompOrEmpty::NonEmpty(fullcomp),mid},left,right)
            },
            None=>{
                //We don't want to return here since we still want to populate the whole tree!
                (Node2{fullcomp:FullCompOrEmpty::Empty(),mid:&mut []},&mut [] as &mut [T],&mut [] as &mut [T]) //TODO rust should make this easier
            }
        };

        let splitter=if !dlevel.should_switch_to_sequential(Depth(depth)){
            let splitter2=&mut splitter2;

            let af= move || {
                self::recurse_rebal2(div_axis.next(),dlevel,left,nodes,sorter,splitter,depth+1,height);
                (splitter,nodes)
            };
            let bf= move || {
                let mut nodes2=SmallVec::new();
                nodes2.push(node);
                self::recurse_rebal2(div_axis.next(),dlevel,right,&mut nodes2,sorter,splitter2,depth+1,height);
                nodes2
            };
            let ((splitter,nodes),mut nodes2)=rayon::join(af,bf);
            for a in nodes2.drain(){
                nodes.push(a);
            }
            splitter
        }else{
            self::recurse_rebal2(div_axis.next(),dlevel.into_seq(),left,nodes,sorter,splitter,depth+1,height);
            nodes.push(node);
            self::recurse_rebal2(div_axis.next(),dlevel.into_seq(),right,nodes,sorter,&mut splitter2,depth+1,height);
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


pub fn recurse_rebal3<'a,A:AxisTrait,T:HasAabb+Send,JJ:par::Joiner,K:Splitter+Send>(
    div_axis:A,
    dlevel:JJ,
    rest:&'a mut [T],
    nodes:&mut Vec<Node2<'a,T>>,
    sorter:impl Sorter,
    splitter:&mut K,
    depth:usize,
    height:usize){
    splitter.node_start();



    if depth<height-1{
        

        let mut splitter2=splitter.div();


        let (node,left,right)=match construct_non_leaf(BinStrat::LeftMidRight,sorter,div_axis,rest){
            Some((fullcomp,left,mid,right))=>{
                
                (Node2{fullcomp:FullCompOrEmpty::NonEmpty(fullcomp),mid},left,right)
            },
            None=>{
                //We don't want to return here since we still want to populate the whole tree!
                (Node2{fullcomp:FullCompOrEmpty::Empty(),mid:&mut []},&mut [] as &mut [T],&mut [] as &mut [T]) //TODO rust should make this easier
            }
        };

        let splitter=if !dlevel.should_switch_to_sequential(Depth(depth)){
            let splitter2=&mut splitter2;

            let af= move || {
                self::recurse_rebal3(div_axis.next(),dlevel,left,nodes,sorter,splitter,depth+1,height);
                (splitter,nodes)
            };
            let bf= move || {
                let mut nodes2=Vec::new();
                nodes2.push(node);
                self::recurse_rebal3(div_axis.next(),dlevel,right,&mut nodes2,sorter,splitter2,depth+1,height);
                nodes2
            };
            let ((splitter,nodes),mut nodes2)=rayon::join(af,bf);
            nodes.append(&mut nodes2);
            splitter
        }else{
            self::recurse_rebal3(div_axis.next(),dlevel.into_seq(),left,nodes,sorter,splitter,depth+1,height);
            nodes.push(node);
            self::recurse_rebal3(div_axis.next(),dlevel.into_seq(),right,nodes,sorter,&mut splitter2,depth+1,height);
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
pub enum BinStrat{
    LeftMidRight,
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
        BinStrat::MidLeftRight=>{
            oned::bin_middle_left_right(div_axis,&med,bots)
        },
        BinStrat::LeftRightMid=>{
            oned::bin_left_right_middle(div_axis,&med,bots)
        }
    };

    debug_assert!(binned.middle.len()!=0);
    
    //We already know that the middile is non zero in length.
    let container_box=dinotree_inner::create_cont(div_axis,binned.middle).unwrap();
    
    sorter.sort(div_axis.next(),binned.middle);
    let full=FullComp{div:med,cont:container_box};
    Some((full,binned.left,binned.middle,binned.right))
}

