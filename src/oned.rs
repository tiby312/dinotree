use inner_prelude::*;
use HasAabb;



///The results of the binning process.
pub struct Binned<'a,T:'a>{
    pub middle:&'a mut [T],
    pub left:&'a mut [T],
    pub right:&'a mut [T],
}


/*
#[cfg(test)]
mod test{
    use test_support::*;
    use test_support;
    use support::*;
    use test::black_box;
    use test::Bencher;
    use oned::*;
    use axgeom;
    struct Bot{
        id:usize,
        //_stuff:[f32;8]
    }


    pub struct Cont<'b,T:'b>{
        pub a:&'b mut T
    }

    impl<'b,T:'b+HasAabb+Send> HasAabb for Cont<'b,T>{
        type Inner=T::Inner;
        type Num=T::Num;

        ///Destructure into the bounding box and mutable parts.
        fn get_mut<'a>(&'a mut self)->(&'a AABBox<T::Num>,&'a mut Self::Inner){
            self.a.get_mut()
        }

        ///Destructue into the bounding box and inner part.
        fn get<'a>(&'a self)->(&'a AABBox<T::Num>,&'a Self::Inner){
            self.a.get()
        }
    }

    #[bench]
    fn bench_pdqselect(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }

        let mut pointers:Vec<Cont<BBox<Numisize,Bot>>>=Vec::with_capacity(bots.len());
        for k in bots.iter_mut(){
            pointers.push(Cont{a:k});
        }
        

        
        b.iter(||{
            let div_axis=axgeom::XAXIS;
            let closure = |a: &Cont<BBox<Numisize,Bot>>, b: &Cont<BBox<Numisize,Bot>>| -> std::cmp::Ordering {

                let arr=(a.get().0).0.get_range(div_axis);
                let brr=(b.get().0).0.get_range(div_axis);
          
                if arr.left() > brr.left(){
                    return std::cmp::Ordering::Greater;
                
                }
                std::cmp::Ordering::Less
            };

            let k={
                let mm=pointers.len()/2;
                pdqselect::select_by(&mut pointers, mm, closure);
                &pointers[mm]
            };

            black_box(k);
        });
        
    }

    #[bench]
    fn bench_pdqselect_no_ind(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }

        
        b.iter(||{
            let div_axis=axgeom::XAXIS;
            let closure = |a: &BBox<Numisize,Bot>, b: &BBox<Numisize,Bot>| -> std::cmp::Ordering {

                let arr=(a.get().0).0.get_range(div_axis);
                let brr=(b.get().0).0.get_range(div_axis);
          
                if arr.left() > brr.left(){
                    return std::cmp::Ordering::Greater;
                
                }
                std::cmp::Ordering::Less
            };

            let k={
                let mm=bots.len()/2;
                pdqselect::select_by(&mut bots, mm, closure);
                &bots[mm]
            };

            black_box(k);
        });
        
    }

    /*
    #[bench]
    fn bench_bin_par(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id,_stuff:[0.0;8]},k)); 
        }


        
        b.iter(||{
            black_box(bin_par::<axgeom::XAXIS_S,_>(&Numisize(500),&mut bots));
        });
        
    }
    */

    #[bench]
    fn bench_sort_no_ind(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }
        

        b.iter(||{
            let div_axis=axgeom::XAXIS;
            let closure = |a: &BBox<Numisize,Bot>, b: &BBox<Numisize,Bot>| -> std::cmp::Ordering {

                let arr=(a.get().0).0.get_range(div_axis);
                let brr=(b.get().0).0.get_range(div_axis);
          
                if arr.left() > brr.left(){
                    return std::cmp::Ordering::Greater;
                
                }
                std::cmp::Ordering::Less
            };

            bots.sort_unstable_by(closure);
            /*
            let k={
                let mm=bots.len()/2;
                pdqselect::select_by(&mut bots, mm, closure);
                &bots[mm]
            };
            */
            black_box(&bots);
        });
        
    }


    #[bench]
    fn bench_sort(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }

        let mut pointers:Vec<Cont<BBox<Numisize,Bot>>>=Vec::with_capacity(bots.len());
        for k in bots.iter_mut(){
            pointers.push(Cont{a:k});
        }
        

        
        b.iter(||{
            let div_axis=axgeom::XAXIS;
            let closure = |a: &Cont<BBox<Numisize,Bot>>, b: &Cont<BBox<Numisize,Bot>>| -> std::cmp::Ordering {

                let arr=(a.get().0).0.get_range(div_axis);
                let brr=(b.get().0).0.get_range(div_axis);
          
                if arr.left() > brr.left(){
                    return std::cmp::Ordering::Greater;
                
                }
                std::cmp::Ordering::Less
            };

            pointers.sort_unstable_by(closure);

            black_box(&pointers);
        });
        
    }

    #[bench]
    fn bench_bin_no_ind(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }
        

        b.iter(||{
            black_box(bin::<axgeom::XAXIS_S,_>(&Numisize(500),&mut bots));
        });
        
    }

    #[bench]
    fn bench_bin(b:&mut Bencher){

        let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

        let mut bots=Vec::new();
        for id in 0..100000{
            let ppp=p.random_point();
            let k=test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(Bot{id},k)); 
        }

        let mut pointers:Vec<Cont<BBox<Numisize,Bot>>>=Vec::with_capacity(bots.len());
        for k in bots.iter_mut(){
            pointers.push(Cont{a:k});
        }
        

        b.iter(||{
            black_box(bin::<axgeom::XAXIS_S,_>(&Numisize(500),&mut pointers));
        });
        
    }
}
*/

/*
pub fn bin_par<'a,'b,A:AxisTrait,X:HasAabb+'b>(med:&X::Num,bots:&'b mut [X])->Binned<'b,X>{
    let ff1=tools::create_empty_slice_at_start_mut(bots);
    let ff2=tools::create_empty_slice_at_start_mut(bots);
    let ff3=tools::create_empty_slice_at_start_mut(bots);
    
    let chunks:Vec<&'b mut [X]>=bots.chunks_mut(4000).collect();
    
    let bins:Vec<Binned<'b,X>>=chunks.into_par_iter().map(|a:&'b mut [X]|bin::<A,_>(med,a)).collect();
    
    let mut last=Binned{middile:ff1,left:ff2,right:ff3};

    for i in bins.into_iter(){
        last=merge::<A,_>(last,i);
    }
    last
}


fn merge<'a,A:AxisTrait,X:HasAabb+'a>(a:Binned<'a,X>,b:Binned<'a,X>)->Binned<'a,X>{
    //assert!(tools::slice_adjacent(a.right,b.middile));

    let amed_len=a.middile.len();
    let aleft_len=a.left.len();
    let aright_len=a.right.len();
    


    let bmed_len=b.middile.len();
    let bleft_len=b.left.len();
    let bright_len=b.right.len();

    let new_med_len=a.middile.len()+b.middile.len();
    let new_left_len=a.left.len()+b.left.len();
    //let new_right_len=a.right.len()+b.right.len();

    let a_slice=tools::join_mut(tools::join_mut(a.middile,a.left),a.right);
    let a_slice_len=a_slice.len();
    let b_slice=tools::join_mut(tools::join_mut(b.middile,b.left),b.right);
    //let b_slice_len=b_slice.len();
    let total=tools::join_mut(a_slice,b_slice);

    //let mut total_med=amed;
    //let mut rr_med=b_slice_len;
    
    //swap_slices(total,amed_len,b_slice_len,bmed_len);
    //let amed_len=amed_len+bmed_len;
    
    //   |    amiddile   |#    aleft    |   aright     |
    //                                                 |#   bmiddile   |   bleft   |    bright   |
    //                    ^                            ^
    //                  target                      b_counter

    let mut b_counter=a_slice_len;
    let mut target=amed_len;
    //append b middiles and shift aleft and aright
    for _ in 0..bmed_len{
        //total.swap(amed_len,amed_len+aleft_len);
        total.swap(b_counter,target);
        total.swap(b_counter,target+aleft_len);
        target+=1;
        b_counter+=1;
    }
    
    //   |    amiddile   |  bmiddile   |#  aleft    |   aright     |
    //                                                             |#  bleft   |    bright   |
    //                                  ^                           ^
    //                                target                    b_counter
    
    target=target+aleft_len;
    
    //   |    amiddile   |  bmiddile   |  aleft    |#   aright     |
    //                                                             |#  bleft   |    bright   |
    //                                              ^               ^
    //                                           target          b_counter
    
    for _ in 0..bleft_len{
        total.swap(b_counter,target);
        total.swap(b_counter,target+aright_len);
        target+=1;
        b_counter+=1;
    }


    //   |    amiddile   |  bmiddile   |  aleft    | bleft  |#   aright     |
    //                                                                      |#    bright   |
    //                                                       ^               ^
    //                                                     target          b_counter
    
    assert_eq!(target,total.len()-bright_len-aright_len);
    assert_eq!(b_counter,total.len()-bright_len);

    let (rest,right)=total.split_at_mut(new_med_len+new_left_len);
    let (middile,left)=rest.split_at_mut(new_med_len);
    Binned{middile,left,right}
}
*/



/*
/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile<left<right
pub fn bin_left_mid_right<'a,'b,A:AxisTrait,X:HasAabb>(med:&X::Num,bots:&'b mut [X])->Binned<'b,X>{
    let bot_len=bots.len();
        
    let mut left_end=0;
    let mut middile_end=0;
    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    //     |   left |   middile|----right-----|-------------|
    //                                         ^
    //                                     index_at
    //
    for index_at in 0..bot_len{
       
        match Accessor::<A>::get(bots[index_at].get()).left_or_right_or_contain(med){
            
            std::cmp::Ordering::Equal=>{
                bots.swap(index_at,middile_end);
                //swap_unchecked(bots,index_at,middile_end);
                middile_end+=1;                    

            },
            std::cmp::Ordering::Less=>{
              
            },
            std::cmp::Ordering::Greater=>{
                bots.swap(index_at,middile_end);
                bots.swap(middile_end,left_end);
                //swap_unchecked(bots,index_at,middile_end);
                //swap_unchecked(bots,middile_end,left_end);
                left_end+=1;
                middile_end+=1;                   
            }
        }
   
    }
    //assert!(index_at==right_start);

    let (rest,right)=bots.split_at_mut(middile_end);
    let (left,middile)=rest.split_at_mut(left_end);
    debug_assert!(left.len()+right.len()+middile.len()==bot_len);
    //debug_assert!(bot_len==index_at,"{:?} ,{:?}",bot_len,index_at);

    Binned{left:left,middile:middile,right:right}
}
*/

#[test]
fn test_binning(){
    use test_support::*;
    use support::*;


    let mut p=PointGenerator::new(&test_support::make_rect((0,1000),(0,1000)),&[100,42,6]);

    struct BBot(BBox<isize,Bot>);
    impl HasAabb for BBot{
        type Num=isize;
        fn get(&self)->&axgeom::Rect<Self::Num>{
            &self.0.rect.0
        }
    }

    let mut bots=Vec::new();
    for id in 0..100000{
        let ppp=p.random_point();
        let k=test_support::create_rect_from_point(ppp);
        bots.push(BBot(BBox::new(Bot{id,col:Vec::new()},k))); 
    }

    let div=500;
    let binned=bin_middile_left_right::<axgeom::XAXISS,_>(&div,&mut bots);

    for b in binned.left{
        assert!(b.0.rect.0.get_range(axgeom::XAXISS::get()).end<div);
    }

    for b in binned.right{
        assert!(b.0.rect.0.get_range(axgeom::XAXISS::get()).start>div);
    }

    for b in binned.middile{
        let r=b.0.rect.0.get_range(axgeom::XAXISS::get());
        assert!(r.start<=div && r.end>=div);
    }
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile<left<right
pub fn bin_middle_left_right<'a,'b,A:AxisTrait,X:HasAabb>(axis:A,med:&X::Num,bots:&'b mut [X])->Binned<'b,X>{
    let bot_len=bots.len();
        
    let mut left_end=0;
    let mut middle_end=0;
    
    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len{
        
            match bots[index_at].get().as_axis().get(axis).left_or_right_or_contain(med){
                
                //If the divider is less than the bot
                std::cmp::Ordering::Equal=>{
                    //left
                    bots.swap(index_at,left_end);
                    bots.swap(left_end,middle_end);
                    //swap_unchecked(bots,index_at,left_end);
                    //swap_unchecked(bots,left_end,middile_end);
                    middle_end+=1;
                    left_end+=1;  
                },
                //If the divider is greater than the bot
                std::cmp::Ordering::Greater=>{
                    //middile
                    bots.swap(index_at,left_end);
                    //swap_unchecked(bots,index_at,left_end);
                    left_end+=1;
                },
                std::cmp::Ordering::Less=>{
                    //right                    
                }
            }
        
        
    }

    let (rest,right)=bots.split_at_mut(left_end);
    let (middle,left)=rest.split_at_mut(middle_end);
//println!("num_bots={:?}",(left.len(),middile.len(),right.len()));
    
    debug_assert!(left.len()+right.len()+middle.len()==bot_len);
    //debug_assert!(bot_len==index_at,"{:?} ,{:?}",bot_len,index_at);

    Binned{left:left,middle:middle,right:right}
}

/*
#[cfg(test)]
pub fn is_sorted<A:AxisTrait,I:HasAabb>(axis:A,collision_botids:&[I]){
    
    if collision_botids.len()==0{
        return;
    }

    let mut last=&collision_botids[0];
    
    for i in &collision_botids[1..]{
         let (a,b)=(i,last);
         
         let (p1,p2)=(Accessor::<A>::get(&(a.get().0).0).left(),Accessor::<A>::get(&(b.get().0).0).left());
        
        assert!(p1>=p2);
        last=i;
    }
}
*/

///Sorts the bots.
pub fn sweeper_update_leaf<I:HasAabb,A:AxisTrait>(axis:A,values: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
        let (p1,p2)=(a.get().as_axis().get(axis).left,b.get().as_axis().get(axis).left);
        if p1 > p2 {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Less
    };

    for i in 0..values.len() {
        for j in (0..i).rev() {
            unsafe{
                let a=values.get_unchecked_mut(j) as *mut I;
                let b=values.get_unchecked_mut(j+1) as *mut I;
            
                if sclosure(&*a,&*b)==std::cmp::Ordering::Greater {
                    std::ptr::swap(a,b)
                } else {
                    break
                }
            }
        }
    }
}
///Sorts the bots.
pub fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

    let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
        let (p1,p2)=(a.get().as_axis().get(axis).left,b.get().as_axis().get(axis).left);
        if p1 > p2 {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Less
    };
    /*
    if JJ::new().is_parallel(){
        //let p=collision_botids.par_iter_mut();
        //p.par_sort_unstable_by(sclosure);
        struct Bo<'a,I:HasAabb+'a>(&'a mut [I]);

        impl<'a,I:HasAabb+'a> ParallelSliceMut<I> for Bo<'a,I>{
            fn as_parallel_slice_mut(&mut self) -> &mut [I]{
                self.0
            }
        }

        Bo(collision_botids).par_sort_unstable_by(sclosure);

    }else{
        */
        /*
        fn selection_sort<T,B:Ord,F:FnMut(&T)->B>(array: &mut [T],mut func:F) {
            let len = array.len();
            for i in 0..len {

                let min = i+array[i..].iter().enumerate().min_by_key(|x| func(x.1))
                                  .unwrap().0;
                array.swap(min, i);
            }
        }

        //use selection sort to minimize number of swaps since we are sorting
        //large objects.
        let ss=|a:&I|->I::Num{
            let p1=Accessor::<A>::get(a.get()).left();
            p1
        };
        selection_sort(collision_botids,ss);
        */
        
        collision_botids.sort_unstable_by(sclosure);
    //}
    //debug_assert!(Self::assert_sorted(collision_botids,accessor));
}

#[test]
fn selection_sort(){
        fn selection_sort<T,B:Ord,F:FnMut(&T)->B>(array: &mut [T],mut func:F) {
            let len = array.len();
            for i in 0..len {

                let min = i+array[i..].iter().enumerate().min_by_key(|x| func(x.1))
                                  .unwrap().0;
                println!("min={:?}",min);
                array.swap(min, i);
            }
        }
    let mut a=[5,3,2,1,4];
    selection_sort(&mut a,|a|*a);
    assert_eq!(a,[1,2,3,4,5]);    
}
/*
    #[cfg(test)]
    mod test{
        use test_support;
        use test_support::Bot;
        use test_support::create_unordered;
        use super::*;
        use axgeom;
        //use Blee;
        use support::BBox;
        use *;
        use ordered_float::NotNaN;
        #[test]
        fn test_get_section(){
            for _ in 0..100{
                let world=test_support::create_word();
                let axis=axgeom::XAXIS;
                let rr=Range{start:100.0,end:110.0};


                  let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..1000).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();
                
                //let mut vec1:Vec<Bot>=(0..500).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();



                let src:Vec<usize>={
                    let mut src_temp=Vec::new();

                    for a in vec1.iter(){

                        if rr.intersects(a.rect.get_range(axis)){
                            src_temp.push(a.val.id);
                        }
                    
                    }
                    src_temp
                };


                let mut sw=Sweeper::new();
                let a=Blee::new(axis);            
                Sweeper::update(&mut vec1,&a);
            
                /*
                println!("Bots:");
                for b in vec1.iter(){
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                */


                let target=sw.get_section(&mut vec1,&rr,&a);

                match target{
                    Some(x)=>{

                        //Assert that all bots that intersect the rect are somewhere in the list outputted by get_setion().
                        for k in src.iter(){
                            let mut found=false;
                            for j in x.iter(){
                                if *k==j.val.id{
                                    found=true;
                                    break;
                                }
                            }
                            assert!(found);
                        }

                        //Assert that the first bot in the outputted list intersects with get_section().
                        let first=x.first().unwrap();
                        let mut found=false;
                        for j in src.iter(){
                            if first.val.id==*j{
                                found=true;
                                break;
                            }
                        }
                        assert!(found);

                        //Assert that the last bot in the outputted list intersects with get_section(). 
                        let last=&x[x.len()-1];
                        let mut found=false;
                        for j in src.iter(){
                            if last.val.id==*j{
                                found=true;
                                break;
                            }
                        }
                        assert!(found);
                    },
                    None=>{
                        assert!(src.len()==0);
                    }
                }

            } 
        }
        

        #[test]
        fn test_bijective_parallel(){       
            for _ in 0..100{
               let world=test_support::create_word();
                //let mut vec1:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();
                //let mut vec2:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:vec1.len()+a,rect:support::get_random_rect(&world)}).collect();
                 

                  let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();


                  let mut vec2:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(vec1.len()+a);
                    BBox::new(bot,rect)
                }
                    ).collect();


                let axis=axgeom::XAXIS;
                            
                let mut src:Vec<(usize,usize)>={
                    let mut src_temp=Vec::new();

                    for i in vec1.iter(){
                        for j in vec2.iter(){
                            let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,NotNaN<f32>,Bot>)=(i,j);

                            if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
                                src_temp.push(create_unordered(&a.val,&b.val));
                            }
                        }
                    }
                    src_temp
                };

                let mut sw=Sweeper::new();
                let a=Blee::new(axis);
                Sweeper::update(&mut vec1,&a);
                Sweeper::update(&mut vec2,&a);


                let mut val=Vec::new();
                //let rr=world.get_range(axis);

                {
                    let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
                        val.push(create_unordered(cc.a.1,cc.b.1));
                    };
                    let mut bk=BleekSF::new(&mut f);
                    sw.find_bijective_parallel((&mut vec1,&mut vec2),&a,&mut bk);
                }
                src.sort_by(&test_support::compair_bot_pair);
                val.sort_by(&test_support::compair_bot_pair);

                /*
                println!("naive result:\n{:?}",(src.len(),&src));
                println!("sweep result:\n{:?}",(val.len(),&val));

                println!("Bots:");
                for b in vec1{
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                println!();
                
                for b in vec2{
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                */
                assert!(src==val);
            }
        }

        #[test]
        fn test_find(){

            //let world=axgeom::Rect::new(-1000.0,1000.0,-1000.0,1000.0);
            let world=test_support::create_word();

                  let mut vec:Vec<BBox<NotNaN<f32>,Bot>>=(0..500).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();
            

            //Lets always order the ids smaller to larger to make it easier to look up.
            // let mut map:HashMap<(usize,usize),()>=HashMap::new();
            let mut src:Vec<(usize,usize)>=Vec::new();

            let axis=axgeom::XAXIS;
            for (e,i) in vec.iter().enumerate(){
                for j in vec[e+1..].iter(){
                    let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,Bot>)=(i,j);

                    if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
                        src.push(create_unordered(&a.val,&b.val));
                    }
                }
            }

            let mut sw=Sweeper::new();
            
            let a=Blee::new(axis);
            Sweeper::update(&mut vec,&a);

            let mut val=Vec::new();

            {
                let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
                    val.push(create_unordered(cc.a.1,cc.b.1));
                };
                let mut bk=BleekSF::new(&mut f);
                sw.find(&mut vec,&a,&mut bk);
            }
            src.sort_by(&test_support::compair_bot_pair);
            val.sort_by(&test_support::compair_bot_pair);

            //println!("{:?}",(src.len(),val.len()));
            //println!("{:?}",val);
            assert!(src==val);
        }
    }


*/