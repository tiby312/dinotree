use inner_prelude::*;



pub struct Vistr<'a,N:'a,T:HasAabb+'a>{
    inner:compt::dfs_order::Vistr<'a,Node3<N,T>>
}

impl<'a,N:'a,T:HasAabb+'a> Vistr<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap<'b>(&'b self)->Vistr<'b,N,T>{
        Vistr{inner:self.inner.create_wrap()}
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for Vistr<'a,N,T>{}

impl<'a,N:'a,T:HasAabb+'a> Visitor for Vistr<'a,N,T>{
    type Item=NodeRef<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        let (nn,rest)=self.inner.next();
        
        let k=match rest{
            Some(((),left,right))=>{
                let f=match &nn.fullcomp{
                    FullCompOrEmpty::NonEmpty(f)=>{
                        Some(f)
                    },
                    FullCompOrEmpty::Empty()=>{
                        None
                    }
                };
                Some((f,Vistr{inner:left},Vistr{inner:right}))
            },
            None=>{
                None
            }
        };

        let j=NodeRef{misc:&nn.n,range:unsafe{nn.mid.as_ref()}};
        (j,k)


    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}





/// Tree Iterator that returns a reference to each node.
/// It also returns the non-leaf specific data when it applies.
pub struct VistrMut<'a,N:'a,T:HasAabb+'a>{
    inner:compt::dfs_order::VistrMut<'a,Node3<N,T>>
}

impl<'a,N:'a,T:HasAabb+'a> VistrMut<'a,N,T>{
    ///It is safe to borrow the iterator and then produce mutable references from that
    ///as long as by the time the borrow ends, all the produced references also go away.
    pub fn create_wrap_mut<'b>(&'b mut self)->VistrMut<'b,N,T>{
        VistrMut{inner:self.inner.create_wrap_mut()}
    }
}

unsafe impl<'a,N:'a,T:HasAabb+'a> compt::FixedDepthVisitor for VistrMut<'a,N,T>{}
impl<'a,N:'a,T:HasAabb+'a> Visitor for VistrMut<'a,N,T>{
    type Item=NodeRefMut<'a,N,T>;
    type NonLeafItem=Option<&'a FullComp<T::Num>>;
    
    fn next(self)->(Self::Item,Option<(Self::NonLeafItem,Self,Self)>){
        let (nn,rest)=self.inner.next();
        
        let k=match rest{
            Some(((),left,right))=>{
                let f=match &nn.fullcomp{
                    FullCompOrEmpty::NonEmpty(f)=>{
                        Some(f)
                    },
                    FullCompOrEmpty::Empty()=>{
                        None
                    }
                };
                Some((f,VistrMut{inner:left},VistrMut{inner:right}))
            },
            None=>{
                None
            }
        };

        let j=NodeRefMut{misc:&mut nn.n,range:unsafe{nn.mid.as_mut()}};
        (j,k)


    }
    fn level_remaining_hint(&self)->(usize,Option<usize>){
        self.inner.level_remaining_hint()
    }
}


pub struct Node3<N,T:HasAabb>{ 
    pub n:N,
    //If this is a non leaf node, then,
    //  div is None iff this node and children nodes do not have any bots in them.
    // Also note, that it is impossible for a node to not have any bots in it but for its decendants to have bots in it.
    // This is because we specifically pick the median.
    // If it is a leaf node, then div being none still means it could have bots in it.
    pub fullcomp:FullCompOrEmpty<T::Num>,
    pub mid:std::ptr::Unique<[T]>
}

pub struct DinoTree<A,N,T:HasAabb>{
    axis:A,
    bots:Vec<T>,
    nodes:compt::dfs_order::CompleteTree<Node3<N,T>>,
}

impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{
    pub fn vistr_mut(&mut self)->VistrMut<N,T>{
        VistrMut{inner:self.nodes.vistr_mut()}
    }
    pub fn vistr(&self)->Vistr<N,T>{
        Vistr{inner:self.nodes.vistr()}
    }

    pub fn height(&self)->usize{
        self.nodes.get_height()
    }
    pub fn num_nodes(&self)->usize{
        self.nodes.get_nodes().len()
    }
    pub fn axis(&self)->A{
        self.axis
    }
    pub fn num_bots(&self)->usize{
        self.bots.len()
    }
}
impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{
    pub fn new<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
        rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->(DinoTree<A,N,BBox<Num,T>>,Vec<u32>)
    {   
        let num_bots=bots.len();
        let max=std::u32::MAX;
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let conts=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
                    });



        let mut conts:Vec<_>=conts.collect();
        
        //let mut nodes=Vec::new();
        let nodes=match rebal_type{
            RebalStrat::First=>{
                let mut nodes=Vec::new();
                tree::recurse_rebal1(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes
            },
            RebalStrat::Second=>{
                let mut nodes=SmallVec::new();
                tree::recurse_rebal2(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes.into_vec()
            },
            RebalStrat::Third=>{
                let mut nodes=Vec::new();
                tree::recurse_rebal3(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);
                nodes
            }
        };
        //tree::recurse_rebal1(axis,par,&mut conts,&mut nodes,sorter,ka,0,height);


        let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();

        let mut new_bots:Vec<BBox<Num,T>>=Vec::with_capacity(num_bots);
        for node in tree.dfs_inorder_iter(){
            for a in node.mid.iter(){
                new_bots.push(BBox{rect:a.rect,inner:bots[a.index as usize]});
            }
        }



        let new_nodes={
            let mut rest:Option<&mut [BBox<Num,T>]>=Some(&mut new_bots);
            let mut new_nodes=Vec::new();
            for node in tree.dfs_inorder_iter(){
                let (b,rest2)=rest.take().unwrap().split_at_mut(node.mid.len());
                rest=Some(rest2);
                
                let b=unsafe{std::ptr::Unique::new_unchecked(b as *mut [_])};
                new_nodes.push(Node3{n,fullcomp:node.fullcomp,mid:b});
            }
            new_nodes
        };

        let tree2=compt::dfs_order::CompleteTree::from_vec(new_nodes,height).unwrap();


        let mut nodes=tree.into_nodes();

        let mover={
            let mut mover=Vec::with_capacity(num_bots);
            for node in nodes.drain(..){
                mover.extend(node.mid.iter().map(|a|a.index));
            }
            mover
        };
        (DinoTree{axis,bots:new_bots,nodes:tree2},mover)    
    }
}
