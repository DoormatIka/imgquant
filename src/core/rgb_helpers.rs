
use std::ops::{Add, AddAssign, Div, Mul, Sub};
use image::Rgb;
use num_traits::{Bounded, Float, Num, NumCast, PrimInt};

#[derive(Copy, Clone, Debug)]
pub struct IRgb<T>(pub Rgb<T>);
impl<T> IRgb<T>
where 
    T: NumCast + Bounded + Copy,
{
    pub fn from_array(arr: [T; 3]) -> Self {
        IRgb(Rgb(arr))
    }
    pub fn get_inner(&self) -> Rgb<T> {
        self.0.clone()
    }
    pub fn color_diff<K: NumCast + Bounded + Copy>(self, rhs: IRgb<K>) -> u32 {
        let lhs: [i32; 3] = self.0.0.map(|x| <i32 as NumCast>::from(x).unwrap());
        let rhs: [i32; 3] = rhs.0.0.map(|x| <i32 as NumCast>::from(x).unwrap());

        let delta = [
            lhs[0] - rhs[0],
            lhs[1] - rhs[1],
            lhs[2] - rhs[2],
        ];

        (3 * delta[0] * delta[0] + 6 * delta[1] * delta[1] + delta[2] * delta[2]) as u32
    }
    pub fn safe_cast<Fro>(value: IRgb<Fro>) -> Option<IRgb<T>> where 
        Fro: NumCast + Copy + Clone,
        // To: NumCast + Bounded,
    {
        // todo: clamp values!
        todo!("Clamp the values first.");
        let value = value.0.0;
        let mut result = [T::min_value(), T::min_value(), T::min_value()];
        for i in 0..3 {
            let v = value[i];
            result[i] = <T as NumCast>::from(v)?;
        }

        Some(IRgb(Rgb(result)))
    }

    pub fn float_to_int<Fro, To>(value: IRgb<Fro>) -> Option<IRgb<To>> where 
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

    pub fn int_to_float<Fro, To>(value: IRgb<Fro>) -> Option<IRgb<To>> where 
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

impl<T> AddAssign for IRgb<T>
where 
    T: AddAssign
{
    fn add_assign(&mut self, rhs: Self) {
        let [sr, sg, sb] = &mut self.0.0;
        let [r, g, b] = rhs.0.0;
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
        let [sr, sg, sb] = &mut self.0.0;
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
        let [sr, sg, sb] = self.0.0;
        let [r, g, b] = rhs.0.0;
        IRgb {
            0: Rgb::<T>([sr + r, sg + g, sb + b]),
        }
    }
}

impl<T> Add<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.0.0;
        IRgb(Rgb([r + rhs, g + rhs, b + rhs]))
    }
}

impl<T> Sub for IRgb<T>
where 
    T: Sub<Output = T>
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.0.0;
        let [r, g, b] = rhs.0.0;
        IRgb(Rgb::<T>([sr - r, sg - g, sb - b]))
    }
}

impl<T> Sub<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.0.0;
        IRgb(Rgb([r - rhs, g - rhs, b - rhs]))
    }
}

impl<T> Mul for IRgb<T>
where 
    T: Mul<Output = T>
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.0.0;
        let [r, g, b] = rhs.0.0;
        IRgb(Rgb::<T>([sr * r, sg * g, sb * b]))
    }
}

impl<T> Mul<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.0.0;
        IRgb(Rgb([r - rhs, g - rhs, b - rhs]))
    }
}

impl<T> Div for IRgb<T>
where 
    T: Div<Output = T>
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.0.0;
        let [r, g, b] = rhs.0.0;
        IRgb(Rgb::<T>([sr / r, sg / g, sb / b]))
    }
}

impl<T> Div<T> for IRgb<T>
where
    T: Copy + Num,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let [r, g, b] = self.0.0;
        IRgb(Rgb([r / rhs, g / rhs, b / rhs]))
    }
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
