
use std::{ops::{Add, AddAssign, Div, Mul, Sub}, u8};
use image::Rgb;

#[derive(Debug, Clone)]
pub struct IRgb<T> {
    inner: Rgb::<T>
}

impl<T> IRgb<T> 
where 
    T: Copy + Add,
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
        i32: From<T> + From<R>
    {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rgb.inner.0;
        let delta_r = i32::from(sr) - i32::from(r);
        let delta_g = i32::from(sg) - i32::from(g);
        let delta_b = i32::from(sb) - i32::from(b);

        (3 * delta_r * delta_r + 6 * delta_g * delta_g + delta_b * delta_b) as u32
    }
}

// plan: let rgb = IRgb::<u32>::from(IRgb::<u8>::new())
impl<Fr, To> From<IRgb<Fr>> for IRgb<To>
where 
    To: From<Fr>,
    Fr: Copy,
{
    fn from(value: IRgb<Fr>) -> Self {
        let [r, g, b] = value.inner.0;
        IRgb { inner: Rgb([To::from(r), To::from(g), To::from(b)]) }
    }
}


impl<T: AddAssign> AddAssign for IRgb<T> {
    fn add_assign(&mut self, rhs: Self) {
        let [sr, sg, sb] = &mut self.inner.0;
        let [r, g, b] = rhs.inner.0;
        *sr += r;
        *sg += g;
        *sb += b;
    }
}

impl<T: Add<Output = T>> Add for IRgb<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([
                sr + r,
                sg + g,
                sb + b,
            ]),
        }
    }
}

impl<T: Sub<Output = T>> Sub for IRgb<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([
                sr - r,
                sg - g,
                sb - b,
            ]),
        }
    }
}

impl<T: Mul<Output = T>> Mul for IRgb<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([
                sr * r,
                sg * g,
                sb * b,
            ]),
        }
    }
}

impl<T: Div<Output = T>> Div for IRgb<T> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let [sr, sg, sb] = self.inner.0;
        let [r, g, b] = rhs.inner.0;
        IRgb {
            inner: Rgb::<T>([
                sr / r,
                sg / g,
                sb / b,
            ]),
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
