// note that instead of using bytemuck, we use an unsafe function (from GiGD)
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((p as *const T) as *const u8,
                                   std::mem::size_of::<T>()) }
}

pub unsafe fn vec_as_u8_slice<T: Sized>(p: &Vec<T>) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((p.as_ptr() as *const T) as *const u8,
                                   std::mem::size_of::<T>() * p.len()) }
}