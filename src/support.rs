use inner_prelude::*;
use ordered_float::NotNaN;

///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>=5
    }
}



///A convenience wrapper that implements the NumTrait around any number that implements the 
///required traits for a NumTrait.
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct NumWrapper<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default>(pub T);
impl<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default> NumTrait for NumWrapper<T>{}

///A premade f32 wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf32(pub NotNaN<f32>);
impl NumTrait for Numf32{}

impl Numf32{
    pub fn from_f32(a:f32)->Numf32{
        Numf32(NotNaN::new(a).unwrap())
    }
}

///A premade f64 wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf64(pub NotNaN<f64>);
impl NumTrait for Numf64{}

///A premade isize wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numisize(pub isize);
impl NumTrait for Numisize{}

///A premade usize wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numusize(pub usize);
impl NumTrait for Numusize{}



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
impl<Nu:NumTrait,T:Send+Sync+Copy> Copy for BBox<Nu,T>{

}
impl<Nu:NumTrait,T:Send+Sync+Clone> Clone for BBox<Nu,T>{
    fn clone(&self)->BBox<Nu,T>{
        BBox{rect:self.rect,val:self.val.clone()}
    }
}
impl<Nu:NumTrait,T:Send+Sync> BBox<Nu,T>{

    #[inline(always)]
    pub fn new(val:T,r:AABBox<Nu>)->BBox<Nu,T>{
        BBox{rect:r,val:val}
    }
}

