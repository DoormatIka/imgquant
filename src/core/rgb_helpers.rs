
use std::{ops::{Add, AddAssign, Div, Mul, Sub}, process::Output};
use image::Rgb;
use num_traits::{Float, NumCast, PrimInt};

struct IRgb<T>(Rgb<T>);
impl<T> IRgb<T>
where 
    T: PrimInt + Float + Clone,
{
    fn float_to_int<Fro, To>(value: IRgb<Fro>) -> Option<IRgb<To>> where 
        Fro: Float,
        To: PrimInt + NumCast,
    {
        let [r, g, b] = value.0.0;
        let clr = IRgb(Rgb([
            NumCast::from(r)?,
            NumCast::from(g)?,
            NumCast::from(b)?,
        ]));

        Some(clr)
    }

    fn int_to_float<Fro, To>(value: IRgb<Fro>) -> Option<IRgb<To>> where 
        Fro: PrimInt,
        To: Float + NumCast,
    {
        let [r, g, b] = value.0.0;
        let clr = IRgb(Rgb([
            NumCast::from(r)?,
            NumCast::from(g)?,
            NumCast::from(b)?,
        ]));

        Some(clr)
    }
}
impl<T> Add for IRgb<T>
where 
    T: Add<T, Output = T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let [r, g, b] = self.0.0;
        let [rhr, rhg, rhb] = rhs.0.0;

        Self(Rgb([
            r + rhr,
            g + rhg,
            b + rhb,
        ]))
    }
}

pub fn color_diff<L, R>(lhs: &Rgb<L>, rhs: &Rgb<R>) -> u32 
where 
    L: Copy,
    R: Copy,
    i32: From<L> + From<R>,
{
    let lhs = lhs.0.map(|x| <i32 as From<L>>::from(x));
    let rhs = rhs.0.map(|x| <i32 as From<R>>::from(x));

    let delta_r = lhs[0] - rhs[0];
    let delta_g = lhs[1] - rhs[1];
    let delta_b = lhs[2] - rhs[2];

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
