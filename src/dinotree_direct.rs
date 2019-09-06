


use crate::tree::*;
use crate::inner_prelude::*;

use crate::dinotree::DinoTreeInner;


///Version of dinotree that makes a copy of all the elements.
pub struct DinoTreeDirect<A: AxisTrait, N:NumTrait,T> {
 	pub(crate) tree:DinoTreeInner<A,BBox<N,T>>,
    pub(crate) rev:Vec<u32>
}


impl<A:AxisTrait,N:NumTrait,T:Copy> DinoTreeDirect<A,N,T>{

    pub fn into_inner(self,inner2:&mut Vec<T>){
        
        inner2.reserve(self.num_bots());
        unsafe{
            inner2.set_len(self.num_bots());
        }
        let DinoTreeDirect{tree,mut rev}=self;  	
    	
        for (index,bot) in rev.drain(..).zip(tree.bots.iter()){
            inner2[index as usize]=bot.inner;
        }
        /*
        for a in rev.drain(..).map(|a|tree.bots[a as usize].inner){
            inner2.push(a);
        }
        */
    }

    pub fn get_bots_mut(&mut self)->&mut [BBox<N,T>]{
        &mut self.tree.bots
    }

    pub fn get_bots(&self)->&[BBox<N,T>]{
        &self.tree.bots
    }
}


///Builder for a DinoTree
pub struct DinoTreeDirectBuilder<A: AxisTrait,N:NumTrait,T,F> {
    axis: A,
    bots: Vec<T>,
    pub(crate) aabb_create: F,
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
    _p:PhantomData<N>
}



impl<A: AxisTrait, T: Copy+Send+Sync, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    DinoTreeDirectBuilder<A,  Num,T, F>
{
    
    ///Build in parallel
    #[inline(always)]
    pub fn build_par(&mut self) -> DinoTreeDirect<A,Num,T> {

        let dlevel = compute_default_level_switch_sequential(self.height_switch_seq, self.height);

        
        let mut conts=self.tree_prep();

        let cont_tree = create_tree_par(self.axis,dlevel, &mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,&mut self.bots,cont_tree)
    }
}

impl<A: AxisTrait, N:NumTrait,T:Copy,F: FnMut(&T) -> Rect<N>> DinoTreeDirectBuilder<A, N,T,F> {
    #[inline]
    pub fn new(axis: A, bots2: &mut Vec<T>,aabb_create:F) -> DinoTreeDirectBuilder< A,N,T,F> {
        let mut bots=Vec::new();
        core::mem::swap(&mut bots,bots2);

        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        DinoTreeDirectBuilder {
        	aabb_create,
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
            _p:PhantomData
        }
    }

    #[inline]
    pub fn build_seq(mut self) -> DinoTreeDirect<A,N, T> {

        let mut conts=self.tree_prep();

        let cont_tree = create_tree_seq(self.axis, &mut conts, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);

        Self::tree_finish(self.axis,&mut self.bots,cont_tree)
    }
    
    fn tree_prep(&mut self)->Vec<BBox<N,u32>>{
        
        let aabb_create=&mut self.aabb_create;
        let bots=&mut self.bots;
        bots.iter_mut().enumerate().map(|(index,bot)|BBox::new((aabb_create)(bot),index as u32)).collect()
    }

    fn tree_finish(axis:A,
    	bots:&mut [T],
        mut tree:compt::dfs_order::CompleteTreeContainer<Node<BBox<N,u32>>,
        compt::dfs_order::PreOrder>) -> DinoTreeDirect<A,N,T>{


    	let rev:Vec<u32>=tree.get_nodes_mut().iter_mut().flat_map(|a|a.get_mut().bots.inner.inner.iter_mut()).map(|a|a.inner).collect();


    	let mut bots:Vec<BBox<N,T>>=tree.get_nodes_mut().iter_mut().flat_map(|a|a.get_mut().bots.inner.iter_mut()).map(|a|BBox::new(*a.rect,bots[*a.inner as usize])).collect();



    	let mut kk:Option<&mut [BBox<N,T>]>=Some(&mut bots);
    	let tree:Vec<Node<BBox<N,T>>>=tree.get_nodes_mut().iter_mut().map(|a|
    		{
    			let (range,b) = kk.take().unwrap().split_at_mut(a.get_mut().bots.len());
    			kk=Some(b);
    			let range=tools::Unique::new(ElemSlice::from_slice_mut(range)).unwrap();
    			Node{range,cont:a.cont,div:a.div}
    		}
    	).collect();

    	let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(tree).unwrap();

    	
	    DinoTreeDirect{
        	rev,
        	tree:DinoTreeInner{
        		axis,
        		tree,
        		bots
        	}}
    	
    }
}


impl<A:AxisTrait,N:NumTrait,T> DinoTreeRefTrait for DinoTreeDirect<A,N,T>{
    type Item=BBox<N,T>;
    type Axis=A;
    type Num=N;
    type Inner=T;
    
    fn axis(&self)->Self::Axis{
        self.tree.axis
    }
    fn vistr(&self)->Vistr<Self::Item>{
        
        Vistr {
            inner: self.tree.tree.vistr(),
        }
        
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        self.tree.tree.get_height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        self.tree.tree.get_nodes().len()
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        self.tree.bots.len()
    }

}


impl<'a,A:AxisTrait,N:NumTrait,T> DinoTreeRefMutTrait for DinoTreeDirect<A,N,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.tree.tree.vistr_mut(),
        }
    }
}

