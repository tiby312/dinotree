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

