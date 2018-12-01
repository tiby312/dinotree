#[allow(dead_code)]
pub fn are_adjacent<'a, T1,T2>(first: &'a [T1], second: &'a [T2]) -> bool {
    let fl = first.len();
    if first[fl..].as_ptr() == second.as_ptr() as *const T1 {
        true
    }else{
        false
    }
}

#[allow(dead_code)]
pub fn slice_join_mut<'a, T>(first: &'a mut [T], second: &'a mut [T]) -> &'a mut [T] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() == second.as_mut_ptr() {
        unsafe {
            ::std::slice::from_raw_parts_mut(first.as_mut_ptr(), fl + second.len())
        }
    }
    else {
        panic!("Slices not adjacent");
    }
}

#[allow(dead_code)]
pub fn slice_join_bytes_mut<'a, T>(first: &'a mut [T], second: &'a mut [u8]) -> &'a mut [u8] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() as *mut u8 == second.as_mut_ptr() {
        unsafe {
            ::std::slice::from_raw_parts_mut(first.as_mut_ptr() as *mut u8, fl*std::mem::size_of::<T>() + second.len())
        }
    }
    else {
        panic!("Slices not adjacent");
    }
}
#[allow(dead_code)]
pub fn bytes_join_slice_mut<'a, T>(first: &'a mut [u8], second: &'a mut [T]) -> &'a mut [u8] {
    let fl = first.len();
    if first[fl..].as_mut_ptr() == second.as_mut_ptr() as *mut u8 {
        unsafe {
            ::std::slice::from_raw_parts_mut(first.as_mut_ptr() as *mut u8, fl + second.len()*std::mem::size_of::<T>())
        }
    }
    else {
        panic!("Slices not adjacent");
    }
}
