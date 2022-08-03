use std::f32::consts::PI;

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

#[inline]
pub fn get_angle(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    if dx == 0 && dy == 0 {
        return 0.0;
    }
    (dy as f32).atan2(dx as f32)
}

#[inline]
pub fn get_angle_diff(a1: f32, a2: f32) -> f32 {
    const TWO_PI: f32 = PI * 2.0;
    let mut delta = (a1 - a2).abs();
    if delta > TWO_PI {
        // TODO: i am simply assuming the weird int coercion in the cpp impl is there for flooring
        delta -= TWO_PI * (delta / TWO_PI).floor() as f32
    }
    if delta > PI {
        delta = TWO_PI - delta;
    }
    round_float_10000(delta)
}

#[inline]
fn round_float_10000(f: f32) -> f32 {
    if f < 1000.0 && f > 0.001 {
        (f * 10000.0).floor() / 10000.0
    } else {
        f
    }
}
