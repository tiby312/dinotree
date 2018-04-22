use inner_prelude::*;
use ordered_float;


///A convenience wrapper that implements the NumTrait around any number that implements the 
///required traits for a NumTrait.
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct NumWrapper<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default>(pub T);
impl<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default> NumTrait for NumWrapper<T>{}



impl NumTrait for ordered_float::NotNaN<f32>{}
impl NumTrait for ordered_float::NotNaN<f64>{}
impl NumTrait for ordered_float::OrderedFloat<f32>{}
impl NumTrait for ordered_float::OrderedFloat<f64>{}
impl NumTrait for isize{}
impl NumTrait for i32{}
impl NumTrait for i64{}
impl NumTrait for u32{}
impl NumTrait for u64{}
impl NumTrait for usize{}




///A generic container that implements the kdtree trait.
#[derive(Debug)]
pub struct BBox<Nu:NumTrait,T:Send+Sync>{
    pub rect:AABBox<Nu>,
    pub val:T
}

impl<Nu:NumTrait,T:Send+Sync> SweepTrait for BBox<Nu,T>{
    type Inner=T;
    type Num=Nu;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a AABBox<Nu>,&'a mut Self::Inner){
        (&self.rect,&mut self.val)
    }
    
    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a AABBox<Nu>,&'a Self::Inner){
        (&self.rect,&self.val)
    }
}

impl<Nu:NumTrait,T:Send+Sync+Clone> Clone for BBox<Nu,T>{
    fn clone(&self)->BBox<Nu,T>{
        BBox{rect:self.rect.clone(),val:self.val.clone()}
    }
}
impl<Nu:NumTrait,T:Send+Sync> BBox<Nu,T>{

    #[inline(always)]
    pub fn new(val:T,r:AABBox<Nu>)->BBox<Nu,T>{
        BBox{rect:r,val:val}
    }
}

