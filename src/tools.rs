pub fn duplicate_empty_slice<T>(arr: &mut [T]) -> &mut [T] {
    assert!(arr.is_empty());
    unsafe { std::slice::from_raw_parts_mut(arr.as_mut_ptr(), 0) }
}




///https://doc.rust-lang.org/1.26.2/core/ptr/struct.Unique.html
///
///A Unique that doesnt require rust nightly.
use std::marker::PhantomData;
#[derive(Copy,Clone,Debug)]
pub struct Unique<T: ?Sized>(
    pub std::ptr::NonNull<T>,
    PhantomData<T>
);

unsafe impl<T:?Sized+Send> Send for Unique<T>{}
unsafe impl<T:?Sized+Sync> Sync for Unique<T>{}
impl<T:?Sized> Unique<T>{
    #[inline]
    pub fn new(ptr:*mut T)->Option<Unique<T>>{
        std::ptr::NonNull::new(ptr).map(|a|Unique(a,PhantomData))
    }
    #[inline]
    pub fn as_ptr(&self)->*mut T{
        self.0.as_ptr()
    }
}


/*

#[repr(C)]
pub struct ReprMut<T>{
    pub ptr:*mut T,
    pub size:usize,
}

#[repr(C)]
pub struct Repr<T>{
    pub ptr:*const T,
    pub size:usize,
}
*/

/*
fn rotate_left<'a,T:Copy>(buffer:&'a mut [T],arr:&'a mut [T])->(&'a mut [T],&'a mut [T]){

    let buffer_len=buffer.len();
    let arr_len=arr.len();

    if buffer.len()<arr.len(){
        buffer.copy_from_slice(&arr[arr_len-buffer_len..]);
    }else{
        buffer[..arr_len].copy_from_slice(arr);
    }

    let all=slice_join_mut(buffer,arr);
    let (arr,buffer)=all.split_at_mut(arr_len);

    assert_eq!(buffer.len(),buffer_len);
    assert_eq!(arr.len(),arr_len);

    (arr,buffer)
}

fn rotate_right<'a,T:Copy>(arr:&'a mut [T],buffer:&'a mut [T])->(&'a mut [T],&'a mut [T]){

    let buffer_len=buffer.len();
    let arr_len=arr.len();

    if buffer.len()<arr.len(){
        buffer.copy_from_slice(&arr[..buffer_len]);
    }else{
        buffer[buffer_len-arr_len..].copy_from_slice(arr);
    }
    let all=slice_join_mut(arr,buffer);
    let (buffer,arr)=all.split_at_mut(buffer_len);

    assert_eq!(buffer.len(),buffer_len);
    assert_eq!(arr.len(),arr_len);

    (buffer,arr)
}


fn rotate_slices_right<'a,T:Copy>(left:&'a mut [T],mid:&'a mut [T],right:&'a mut [T],buffer:&'a mut [T])->(&'a mut [T],&'a mut [T],&'a mut [T],&'a mut [T]){
    let (buffer,right)=rotate_right(right,buffer);
    let (buffer,mid)=rotate_right(mid,buffer);
    let (buffer,left)=rotate_right(left,buffer);
    (buffer,left,mid,right)
}
fn rotate_slices_left<'a,T:Copy>(buffer:&'a mut [T],left:&'a mut [T],mid:&'a mut [T],right:&'a mut [T])->(&'a mut [T],&'a mut [T],&'a mut [T],&'a mut [T]){

    let (left,rest)=rotate_left(buffer,left);
    let (mid,rest)=rotate_left(rest,mid);
    let (right,buffer)=rotate_left(rest,right);
    (left,mid,right,buffer)
}



*/
/*
mod chunk{
    use tree_alloc;
    pub struct MemChunk{
        vec:Vec<u8>,
        offset:isize,
        num_bytes:usize
    }
    impl MemChunk{

        pub fn into_inner(self)->Vec<u8>{
            self.vec
        }
        pub fn as_mut_ptr(&mut self)->*mut u8{
            self.vec.as_mut_ptr()
        }
        pub fn get_end_mut_ptr(&mut self)->*mut u8{
            let num_bytes=self.num_bytes;
            unsafe{
                self.vec.as_mut_ptr().offset(num_bytes as isize)
            }
        }
        pub fn capacity(&self)->usize{
            self.num_bytes
        }
        pub fn get_mut(&mut self)->&mut [u8]{
            let offset=self.offset;
            unsafe{
                let a=self.vec.as_mut_ptr().offset(offset);
                std::mem::transmute(tree_alloc::ReprMut{ptr:a,size:self.num_bytes})
            }
        }
        pub fn new(num_bytes:usize,alignment:usize)->MemChunk{


            let (offset,vec)={

                let mut vec=Vec::with_capacity(alignment+num_bytes);


                let mut counter=vec.as_ptr() as *mut u8;



                let offset=counter.align_offset(alignment);
                if offset==usize::max_value(){
                    panic!("Error finding alignment!");
                }


                (offset as isize,vec)
            };

            if num_bytes %alignment!=0{
                panic!("fail!");
            }
            MemChunk{vec,offset,num_bytes}
        }

    }
}
*/

#[allow(dead_code)]
pub fn are_adjacent<'a, T1, T2>(first: &'a [T1], second: &'a [T2]) -> bool {
    let fl = first.len();
    first[fl..].as_ptr() == second.as_ptr() as *const T1
}

#[allow(dead_code)]
pub fn slice_join_mut<'a, T>(first: &'a mut [T], second: &'a mut [T]) -> &'a mut [T] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() == second.as_mut_ptr() {
        unsafe { ::std::slice::from_raw_parts_mut(first.as_mut_ptr(), fl + second.len()) }
    } else {
        panic!("Slices not adjacent");
    }
}

#[allow(dead_code)]
pub fn slice_join_bytes_mut<'a, T>(first: &'a mut [T], second: &'a mut [u8]) -> &'a mut [u8] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() as *mut u8 == second.as_mut_ptr() {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                first.as_mut_ptr() as *mut u8,
                fl * std::mem::size_of::<T>() + second.len(),
            )
        }
    } else {
        panic!("Slices not adjacent");
    }
}
#[allow(dead_code)]
pub fn bytes_join_slice_mut<'a, T>(first: &'a mut [u8], second: &'a mut [T]) -> &'a mut [u8] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() == second.as_mut_ptr() as *mut u8 {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                first.as_mut_ptr() as *mut u8,
                fl + second.len() * std::mem::size_of::<T>(),
            )
        }
    } else {
        panic!("Slices not adjacent");
    }
}
