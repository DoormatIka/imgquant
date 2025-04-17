
use std::{ops::{Add, AddAssign, Div, Mul, Sub}, process::Output, u8};
use image::Rgb;
use num_traits::{Bounded, Num, NumCast, ToPrimitive};

#[derive(Debug, Clone, Copy)]
pub struct IRgb<T> {
    inner: Rgb::<T>
}

pub trait Convert<To>
where 
    To: Clone + Copy + Num
{
    fn lower_convert(self) -> Option<IRgb<To>>;
}
impl<Fr, To> Convert<To> for IRgb<Fr>
where
    To: Clone + Copy + From<Fr> + Num + NumCast + ToPrimitive + PartialOrd,
    Fr: Clone + Copy + Num + NumCast + ToPrimitive + Bounded,
{
    /// Converts the internal values from a higher to lower bit representation.
    fn lower_convert(self) -> Option<IRgb<To>> {
        let [r, g, b] = self.inner.0;
        Some(IRgb::from_array([
            NumCast::from(r)?,
            NumCast::from(g)?,
            NumCast::from(b)?,
        ]))
    }
}

impl<T> IRgb<T> 
where 
    T: Copy,
{
    pub fn new(rgb: Rgb<T>) -> Self {
        IRgb { inner: rgb }
    }
    pub fn from_array(rgb: [T; 3]) -> Self {
        IRgb { inner: Rgb(rgb) }
    }
    pub fn get_inner(&self) -> Rgb<T> {
        self.inner
    }
    pub fn color_diff<R>(&self, rgb: &IRgb<R>) -> u32
    where
        R: Copy,
        i32: From<T> + From<R>,
    {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rgb.inner.0;
        let delta_r = <i32 as From<T>>::from(sr) - <i32 as From<R>>::from(r);
        let delta_g = <i32 as From<T>>::from(sg) - <i32 as From<R>>::from(g);
        let delta_b = <i32 as From<T>>::from(sb) - <i32 as From<R>>::from(b);

        (3 * delta_r * delta_r + 6 * delta_g * delta_g + delta_b * delta_b) as u32
    }
}

impl<T> AddAssign for IRgb<T>
where 
    T: AddAssign
{
    fn add_assign(&mut self, rhs: Self) {
        let [sr, sg, sb] = &mut self.inner.0;
        let [r, g, b] = rhs.inner.0;
        *sr += r;
        *sg += g;
        *sb += b;
    }
}

impl<T> AddAssign<T> for IRgb<T> 
where 
    T: Copy + Num + AddAssign<T>,
{
    fn add_assign(&mut self, rhs: T) {
        let [sr, sg, sb] = &mut self.inner.0;
        *sr += rhs;
        *sg += rhs;
        *sb += rhs;
    }
}

impl<T> Add for IRgb<T>
where 
    T: Add<Output = T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([sr + r, sg + g, sb + b]),
        }
    }
}

impl<T> Add<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.inner.0;
        IRgb {
            inner: Rgb([r + rhs, g + rhs, b + rhs]),
        }
    }
}

impl<T> Sub for IRgb<T>
where 
    T: Sub<Output = T>
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([sr - r, sg - g, sb - b]),
        }
    }
}

impl<T> Sub<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.inner.0;
        IRgb {
            inner: Rgb([r - rhs, g - rhs, b - rhs]),
        }
    }
}

impl<T> Mul for IRgb<T>
where 
    T: Mul<Output = T>
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([sr * r, sg * g, sb * b]),
        }
    }
}

impl<T> Mul<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.inner.0;
        IRgb {
            inner: Rgb([r - rhs, g - rhs, b - rhs]),
        }
    }
}

impl<T> Div for IRgb<T>
where 
    T: Div<Output = T>
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([sr / r, sg / g, sb / b]),
        }
    }
}

impl<T> Div<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.inner.0;
        IRgb {
            inner: Rgb([r / rhs, g / rhs, b / rhs]),
        }
    }
}

/*
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
*/
