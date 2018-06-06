use rayon;
use compt::Depth;

pub trait Joiner:Send+Sync+Copy+Clone{
    fn new(d:Depth)->Self;
    fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB);
    //fn is_parallel(&self)->bool;
    fn into_seq(&self)->Sequential;
    fn should_switch_to_sequential(&self,a:Depth)->bool;
}

#[derive(Copy,Clone)]
pub struct Parallel(pub Depth);
impl Joiner for Parallel{
    fn new(d:Depth)->Self{
      Parallel(d)
    }

    fn into_seq(&self)->Sequential{
      Sequential
    }

    fn should_switch_to_sequential(&self,a:Depth)->bool{
      //Seems like 6 is ideal for my dell xps laptop
      //8 is best on my android phone.
      a.0>=(self.0).0
    }

    fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
      rayon::join(oper_a, oper_b)
    }
}

#[derive(Copy,Clone)]
pub struct Sequential;
impl Joiner for Sequential{
    fn new(_:Depth)->Self{
      Sequential
    }
    fn into_seq(&self)->Sequential{
      Sequential
    }

    fn should_switch_to_sequential(&self,_a:Depth)->bool{
       true
    }

    fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
        let a = oper_a();
        let b = oper_b();
        (a, b)
    }
}