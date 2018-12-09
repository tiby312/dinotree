use inner_prelude::*;

use tree::dinotree_simple;







pub use tree::dinotree_simple::Vistr;
pub use tree::dinotree_simple::VistrMut;



pub struct DinoTree<A:AxisTrait,N,T:HasAabb>{
	inner:dinotree_simple::DinoTree<A,N,T>,
    mover:Vec<u32>
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DinoTree<A,N,BBox<Num,T>>{

    #[inline]
	pub(crate) fn new_inner<JJ:par::Joiner,K:Splitter+Send,F:FnMut(&T)->Rect<Num>>(
	    rebal_type:RebalStrat,axis:A,n:N,bots:&[T],aabb_create:F,ka:&mut K,height:usize,par:JJ,sorter:impl Sorter)->DinoTree<A,N,BBox<Num,T>>
	{   
        let (inner,mover)=dinotree_simple::DinoTree::new(rebal_type,axis,n,bots,aabb_create,ka,height,par,sorter);

        DinoTree{inner,mover}
	}

    
    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    #[inline]
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{  
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;


        //TODO simplify this code!!!
        //See the data project for reasoning behind this value.
        const DEPTH_SEQ:usize=5;

        let gg=if height<=DEPTH_SEQ{
            0
        }else{
            height-DEPTH_SEQ
        };
        
        let dlevel=par::Parallel::new(Depth(gg));

        Self::new_adv(None,axis,n,bots,aabb_create,&mut ka,height,dlevel,DefaultSorter)
    }

    #[inline]
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl FnMut(&T)->Rect<Num>)->DinoTree<A,N,BBox<Num,T>>{   
        let height=advanced::compute_tree_height_heuristic(bots.len()); 
        let mut ka=advanced::SplitterEmpty;
        Self::new_inner(RebalStrat::First,axis,n,bots,aabb_create,&mut ka,height,par::Sequential,DefaultSorter)
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
        let treev=self.inner.bots.iter_mut();
        
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
        self.inner.bots.iter_mut()
    }

    ///See iter_mut
    #[inline]
    pub fn iter(&self)->std::slice::Iter<T>{
        self.inner.bots.iter()
    }
    
    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    #[inline]
    pub fn axis(&self)->A{
        self.inner.axis()
    }

    ///Get the height of the tree.
    #[inline]
    pub fn height(&self)->usize{
        self.inner.height()
    }

    ///Create a mutable tree visitor.
    #[inline]
    pub fn vistr_mut<'b>(&'b mut self)->dinotree_simple::VistrMut<'b,N,T>{
        self.inner.vistr_mut()
    }

    ///Create an immutable tree visitor.
    #[inline]
    pub fn vistr<'b>(&'b self)->dinotree_simple::Vistr<'b,N,T>{
        self.inner.vistr()
    }

    ///Returns the number of bots that are in the tree.
    #[inline]
    pub fn num_bots(&self)->usize{
        self.inner.num_bots()

    }

    ///Returns the number of nodes in the tree.
    #[inline]
    pub fn num_nodes(&self)->usize{
        self.inner.num_nodes()
    }
}

