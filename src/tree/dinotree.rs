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
    pub fullcomp:FullCompOrEmpty<T::Num>,
    pub mid:std::ptr::Unique<[T]>
}




pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
	//inner:dinotree_simple::DinoTree<A,N,T>,
    axis:A,
    bots:Vec<T>,
    nodes:compt::dfs_order::CompleteTree<Node3<N,T>>,
    mover:Vec<u32>
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{


    #[inline]
	pub(crate) fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
	    rebal_type:RebalStrat,axis:A,n:N,bots:&[T],mut aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree<A,N,BBox<Num,T>>
	{   
        //let (inner,mover)=dinotree_simple::DinoTree::new(rebal_type,axis,n,bots,aabb_create,ka,height,par,sorter);


        let num_bots=bots.len();
        let max=std::u32::MAX;
        
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let mut conts:Vec<_>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
        }).collect();



        let mut nodes=Vec::with_capacity(tree::nodes_left(0,height));
        
        match rebal_type{
            RebalStrat::First=>{
                tree::recurse_rebal(axis,par,&mut conts,&mut nodes,sorter,ka,0,height,BinStrat::LeftMidRight);
            },
            RebalStrat::Second=>{
                tree::recurse_rebal(axis,par,&mut conts,&mut nodes,sorter,ka,0,height,BinStrat::MidLeftRight);
            },
            RebalStrat::Third=>{
                tree::recurse_rebal(axis,par,&mut conts,&mut nodes,sorter,ka,0,height,BinStrat::LeftRightMid);
            }
        }

        let tree=compt::dfs_order::CompleteTree::from_vec(nodes,height).unwrap();


        let mut new_bots:Vec<BBox<Num,T>>=Vec::with_capacity(num_bots);
        for node in tree.dfs_inorder_iter(){
            for a in node.mid.iter(){
                new_bots.push(BBox{rect:a.rect,inner:bots[a.index as usize]});
            }
        }


        let new_nodes={
            let mut rest:Option<&mut [BBox<Num,T>]>=Some(&mut new_bots);
            let mut new_nodes=Vec::with_capacity(tree.get_nodes().len());
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
        DinoTree{mover,axis,bots:new_bots,nodes:tree2}
	}

    
    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    #[inline]
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{  
        advanced::new_adv(None,axis,n,bots,aabb_create,None,&mut advanced::SplitterEmpty,None,)
    }

    #[inline]
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        advanced::new_adv_seq(None,axis,n,bots,aabb_create,None,&mut advanced::SplitterEmpty)
    }
    
}


impl<A:AxisTrait,N,T:HasAabb> DinoTree<A,N,T>{
    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    #[inline]
    pub fn apply<X>(&self,bots:&mut [X],conv:impl Fn(&T,&mut X)){
        assert_eq!(bots.len(),self.num_bots());
        for (bot,mov) in self.iter().zip_eq(self.mover.iter()){
            let target=&mut bots[*mov as usize];
            conv(bot,target);
        }
    }

    #[inline]
    pub fn apply_into<X>(&mut self,bots:&[X],conv:impl Fn(&X,&mut T)){
        
        assert_eq!(bots.len(),self.num_bots());

        //let treev=self.inner.nodes.dfs_preorder_iter().flat_map(|(a,_)|a.range.iter_mut());
        let treev=self.bots.iter_mut();
        
        for (bot,mov) in treev.zip_eq(self.mover.iter()){
            let source=&bots[*mov as usize];
            conv(source,bot)
        }
        
    }

    ///Iterate over al the bots in the tree. The order in which they are iterated is dfs in order.
    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    ///But this is useful if you need to iterate over all the bots aabbs.
    #[inline]
    pub fn iter_mut(&mut self)->std::slice::IterMut<T>{
        self.bots.iter_mut()
    }

    ///See iter_mut
    #[inline]
    pub fn iter(&self)->std::slice::Iter<T>{
        self.bots.iter()
    }
    
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

