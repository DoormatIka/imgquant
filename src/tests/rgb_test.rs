
#[cfg(test)]
use image::Rgb;
use crate::core::rgb_helpers::add_colors;
use crate::core::rgb_helpers::div_colors;
use crate::core::rgb_helpers::mul_colors;
use crate::core::rgb_helpers::IRgb;
use crate::core::rgb_helpers::sub_colors;

#[test]
fn test_add() {
    let rgb = IRgb::from_array([100; 3]) + 50;
    let mut r = Rgb([100; 3]);
    add_colors(&mut r, &Rgb([50; 3]));
    assert_eq!(IRgb(r), rgb);
}

#[test]
fn test_sub() {
    let rgb = IRgb::from_array([100; 3]) - 50;
    let s: Rgb<i32> = sub_colors(&Rgb([100; 3]), &Rgb([50; 3]));
    assert_eq!(IRgb(s), rgb);
}

#[test]
fn test_mul() {
    let rgb = IRgb::from_array([100; 3]) * 5;
    let s: Rgb<i32> = mul_colors(&Rgb([100; 3]), &Rgb([5; 3]));
    assert_eq!(IRgb(s), rgb);
}

#[test]
fn test_div() {
    let rgb = IRgb::from_array([100; 3]) / 5;
    let s: Rgb<i32> = div_colors(&Rgb([100; 3]), &Rgb([5; 3]));
    assert_eq!(IRgb(s), rgb);
}

#[test]
fn test_clamp() {
    let mut rgb = IRgb::from_array([100; 3]);
    rgb.clamp(1, 5);
    assert_eq!(rgb, IRgb::from_array([5; 3]));
}
