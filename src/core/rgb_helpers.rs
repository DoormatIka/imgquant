
use std::ops::{AddAssign, Sub, Mul, Div};
use image::Rgb;

pub fn color_diff<L, R>(lhs: &Rgb<L>, rhs: &Rgb<R>) -> u32 
where 
    L: Copy,
    R: Copy,
    i32: From<L> + From<R>,
{
    let delta_r = i32::from(lhs.0[0]) - i32::from(rhs.0[0]);
    let delta_g = i32::from(lhs.0[1]) - i32::from(rhs.0[1]);
    let delta_b = i32::from(lhs.0[2]) - i32::from(rhs.0[2]);

    (3 * delta_r * delta_r + 6 * delta_g * delta_g + delta_b * delta_b) as u32
}

pub fn add_colors<T, U>(color: &mut Rgb<T>, other_color: &Rgb<U>) 
where
    T: AddAssign + From<U>,
    U: Copy,
{
    color.0.iter_mut()
        .zip(other_color.0.iter().copied())
        .for_each(|(a, b)| *a += T::from(b));
}

pub fn sub_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Sub<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] - other_color.0[0],
        color.0[1] - other_color.0[1],
        color.0[2] - other_color.0[2],
    ])
}

pub fn mul_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Mul<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] * other_color.0[0],
        color.0[1] * other_color.0[1],
        color.0[2] * other_color.0[2],
    ])
}

pub fn div_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Div<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] / other_color.0[0],
        color.0[1] / other_color.0[1],
        color.0[2] / other_color.0[2],
    ])
}
