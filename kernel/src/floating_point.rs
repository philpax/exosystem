#[no_mangle]
pub extern "C" fn fmin(x: f64, y: f64) -> f64 {
    core::primitive::f64::min(x, y)
}

#[no_mangle]
pub extern "C" fn fminf(x: f32, y: f32) -> f32 {
    core::primitive::f32::min(x, y)
}

#[no_mangle]
pub extern "C" fn fmax(x: f64, y: f64) -> f64 {
    core::primitive::f64::max(x, y)
}

#[no_mangle]
pub extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    core::primitive::f32::max(x, y)
}

#[no_mangle]
pub extern "C" fn fmod(x: f64, y: f64) -> f64 {
    unsafe { core::intrinsics::frem_fast(x, y) }
}

#[no_mangle]
pub extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    unsafe { core::intrinsics::frem_fast(x, y) }
}
