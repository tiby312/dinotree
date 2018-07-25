 use super::*;


pub struct TreeTimeResultIterator(
    std::vec::IntoIter<f64>
);
impl std::iter::FusedIterator for TreeTimeResultIterator{}
unsafe impl std::iter::TrustedLen for TreeTimeResultIterator{}
impl Iterator for TreeTimeResultIterator{
    type Item=f64;
    fn next(&mut self)->Option<Self::Item>{
        self.0.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.0.size_hint()
    }
}


///Used for debugging performance. The user does not need to worry about this.
///It is exposesed so that the algorithms that operate on this tree may also use this funcionality.
pub trait TreeTimerTrait:Sized+Send{
    type Bag:Send+Sized;
    fn combine(a:Self::Bag,b:Self::Bag)->Self::Bag;
    fn new(height:usize)->Self;
    fn leaf_finish(self)->Self::Bag;
    fn start(&mut self);
    fn next(self)->(Self,Self);
}

///When we do not use debugging, this version is used.
pub struct TreeTimerEmpty;
pub struct BagEmpty;
impl TreeTimerTrait for TreeTimerEmpty{
    type Bag=BagEmpty;
    fn combine(_a:BagEmpty,_b:BagEmpty)->BagEmpty{
        BagEmpty
    }

    fn new(_height:usize)->TreeTimerEmpty{
        TreeTimerEmpty
    }

    fn leaf_finish(self)->BagEmpty{
        BagEmpty
    }

    fn start(&mut self){

    }
    fn next(self)->(Self,Self){
        (TreeTimerEmpty,TreeTimerEmpty)
    }

}
pub struct Bag{
    a:Vec<f64>
}
impl Bag{
    pub fn into_iter(self)->TreeTimeResultIterator{
        TreeTimeResultIterator(self.a.into_iter())
    }
}

///Used when the user wants the time to be returned.
pub struct TreeTimer2{
    a:Vec<f64>,
    index:usize,
    timer:Option<tools::Timer2>
}



impl TreeTimerTrait for TreeTimer2{
    type Bag=Bag;
    fn combine(mut a:Bag,b:Bag)->Bag{
        for (i,j) in a.a.iter_mut().zip(b.a.iter()){
            *i+=j;
        }
        a
    }
    fn new(height:usize)->TreeTimer2{
        let v=(0..height).map(|_|0.0).collect();
        
        TreeTimer2{a:v,index:0,timer:None}
    }

    //Can be called prematurely if there are no children
    fn leaf_finish(self)->Bag{

        let TreeTimer2{mut a,index,timer}=self;
        //debug_assert!(index==a.len()-1);
        a[index]+=timer.unwrap().elapsed();
        Bag{a:a}
    }

    fn start(&mut self){
        self.timer=Some(tools::Timer2::new())
    }

    fn next(self)->(TreeTimer2,TreeTimer2){
        let TreeTimer2{mut a,index,timer}=self;
        a[index]+=timer.unwrap().elapsed();

        let b=(0..a.len()).map(|_|0.0).collect();
        (
            TreeTimer2{a:a,index:index+1,timer:None},
            TreeTimer2{a:b,index:index+1,timer:
                None}
        )
    }

  
}