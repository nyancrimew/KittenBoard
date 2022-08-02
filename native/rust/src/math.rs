#[inline]
pub fn square_float(x: f32) -> f32 {
    x * x
}

#[inline]
pub fn get_distance_int(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    ((x1 - x2) as f32).hypot((y1 - y2) as f32) as i32
}

#[inline]
pub fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> i32 {
    (x1 - x2).hypot(y1 - y2) as i32
}
