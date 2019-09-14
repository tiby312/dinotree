//use crate::dinotree::*;
use crate::inner_prelude::*;
use crate::tree::dinotree_good::*;
/*
///The trait through which algorithms can use the not sorted version of the dinotree
pub trait NotSortedRefTrait{
    type Item:HasInner<Num=Self::Num,Inner=Self::Inner>;
    type Axis:AxisTrait;
    type Num:NumTrait;
    type Inner;
    
    fn axis(&self)->Self::Axis;
    fn vistr(&self)->Vistr<Self::Item>;


    ///Return the height of the dinotree.
    fn height(&self) -> usize;

    ///Return the number of nodes of the dinotree.
    fn num_nodes(&self) -> usize;

    ///Return the number of bots in the tree.
    fn num_bots(&self) -> usize;

}

///The mutable part of the not sorted trait.
pub trait NotSortedRefMutTrait:NotSortedRefTrait{
    fn vistr_mut(&mut self)->VistrMut<Self::Item>;
}
*/


/*
impl<'a,A:AxisTrait,N:NumTrait,T> NotSortedRefTrait for NotSorted<'a,A,N,T>{
    type Item=BBoxMut<'a,N,T>;
    type Axis=A;
    type Num=N;
    type Inner=T;
    
    #[inline(always)]
    fn axis(&self)->Self::Axis{
        self.0.axis()
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<Self::Item>{
        Vistr {
            inner: self.0.inner.tree.vistr(),
        }
    }

    ///Return the height of the dinotree.
    #[inline(always)]
    fn height(&self) -> usize
    {
        self.0.height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline(always)]
    fn num_nodes(&self) -> usize
    {
        self.0.num_nodes()
    }

    ///Return the number of bots in the tree.
    #[inline(always)]
    fn num_bots(&self) -> usize
    {
        self.0.num_bots()
    }

}
*/
/*
impl<'a,A:AxisTrait,N:NumTrait,T> NotSortedRefMutTrait for NotSorted<'a,A,N,T>{    
    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.0.inner.tree.vistr_mut(),
        }
    }
}
*/


