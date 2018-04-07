use std;


use std::time::Instant;

pub struct Timer2{
    a:std::time::Instant
}

impl Timer2{
    pub fn new()->Timer2{
        Timer2{a:Instant::now()}
    }

    ///Returns the time since this object was created in seconds.
    pub fn elapsed(&self)->f64{
        let elapsed = self.a.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        sec
    }
}
