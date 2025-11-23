use std::collections::HashMap;
use rand::Rng;
use std::ops::{Add, AddAssign, Deref, Mul, MulAssign, Sub, SubAssign};

pub const BASIS: u16 = 10000;

#[derive(Debug, Clone, Copy)]
pub enum Modifier<T> {
    Add(T),
    Multiply(f64),
    Set(T),
}

pub trait Modifiable<T> {
    fn modify(&mut self, modifiers: &mut [Modifier<T>]);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Range<T> {
    value: T,
    min: T,
    max: T,
}

#[derive(Debug, Default)]
pub struct BasedRange<T>
where
    T: Clone + Copy + Add + AddAssign + Mul + MulAssign
{
    base: T,
    value: T,
    min: T,
    max: T,
}

impl<T> PartialEq for Modifier<T> {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl<T> Eq for Modifier<T> {}

impl<T> PartialOrd for Modifier<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Modifier<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let rank = |modifier: &Self| -> u8 {
            match modifier {
                Modifier::Add(_) => 0,
                Modifier::Multiply(_) => 1,
                Modifier::Set(_) => 2,
            }
        };

        rank(self).cmp(&rank(other))
    }
}

impl<T> Modifiable<T> for BasedRange<T>
where
    T: Clone + Copy + Add + AddAssign + Mul<f64, Output = T> + MulAssign<f64> + Default
{
    fn modify(&mut self, modifiers: &mut [Modifier<T>]) {
        self.value = self.base;

        // Modify by order of: add -> multiply -> set
        modifiers.sort();

        for modifier in modifiers {
            match modifier {
                Modifier::Add(v) => self.value += *v,
                Modifier::Multiply(v) => self.value *= *v,
                Modifier::Set(v) => self.value = *v,
            }
        }
    }
}



impl<T> BasedValue<T>
where
    T: Clone + Copy,
{
    pub fn new(value: T, base: T) -> Self {
        Self { value, base }
    }

    pub fn reset(&mut self) {
        self.value = self.base;
    }
}

impl<T> Deref for BasedValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> MaxedValue<T>
where
    T: PartialOrd + Clone + Copy + Default,
{
    pub fn new(value: T, max: T) -> Self {
        let value = clamp_max(value, max);
        Self { value, max }
    }

    pub fn set(&mut self, value: T) {
        self.value = clamp_max(value, self.max);
    }
}

impl<T> Deref for MaxedValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> RangedValue<T>
where
    T: PartialOrd + Clone + Copy + Default,
{
    pub fn new(value: T, min: T, max: T) -> Self {
        let value = clamp(value, min, max);
        Self { value, min, max }
    }

    pub fn set(&mut self, value: T) {
        self.value = clamp(value, self.min, self.max);
    }
}

impl<T> Deref for RangedValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AddAssign<T> for RangedValue<T>
where
    T: Add<Output = T> + PartialOrd + Clone + Copy + Default,
{
    fn add_assign(&mut self, rhs: T) {
        self.value = clamp(self.value + rhs, self.min, self.max);
    }
}

impl<T> SubAssign<T> for RangedValue<T>
where
    T: Sub<Output = T> + PartialOrd + Clone + Copy + Default,
{
    fn sub_assign(&mut self, rhs: T) {
        self.value = clamp(self.value - rhs, self.min, self.max);
    }
}

impl<T> Modifier<T> {
    pub fn modify(&self, target: &mut Modifiable<T>) {

    }
}

/// Basis pointed probability value
#[derive(Debug, Default, Clone, Copy)]
pub struct Chance {
    pub value: RangedValue<u16>,
}

impl Chance {
    pub fn new(value: u16) -> Self {
        Self {
            value: RangedValue::new(value, 0, BASIS),
        }
    }

    pub fn hit(&self) -> bool {
        rand::rng().random_range(0..=BASIS) >= *self.value
    }
}

impl Deref for Chance {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// A value bounded by a minimum and a maximum
///
///  If input is less than min then this returns min.
///  If input is greater than max then this returns max.
///  Otherwise, this returns input.
///
/// **Panics** in debug mode if `!(min <= max)`.
#[inline]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    debug_assert!(min <= max);
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// A value bounded by a minimum value
///
///  If input is less than min then this returns min.
///  Otherwise, this returns input.
///  `clamp_min(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::min(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(min == min)`. (This occurs if `min` is `NAN`.)
#[inline]
pub fn clamp_min<T: PartialOrd>(value: T, min: T) -> T {
    debug_assert!(min == min);
    if value < min { min } else { value }
}

/// A value bounded by a maximum value
///
///  If input is greater than max then this returns max.
///  Otherwise, this returns input.
///  `clamp_max(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::max(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(max == max)`. (This occurs if `max` is `NAN`.)
#[inline]
pub fn clamp_max<T: PartialOrd>(value: T, max: T) -> T {
    debug_assert!(max == max);
    if value > max { max } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maxed_value_test() {
        assert_eq!(*MaxedValue::new(1, 1), 1);
        assert_eq!(*MaxedValue::new(10, 1), 1);
        assert_eq!(*MaxedValue::new(1, 10), 1);
    }

    #[test]
    fn ranged_value_test() {
        assert_eq!(*RangedValue::new(1, 1, 10), 1);
        assert_eq!(*RangedValue::new(-1, 1, 10), 1);
        assert_eq!(*RangedValue::new(15, 1, 10), 10);
    }

    #[test]
    fn clamp_test() {
        // Integer tests
        assert_eq!(1, clamp(1, -1, 2));
        assert_eq!(-1, clamp(-2, -1, 2));
        assert_eq!(2, clamp(3, -1, 2));
        assert_eq!(1, clamp_min(1, -1));
        assert_eq!(-1, clamp_min(-2, -1));
        assert_eq!(-1, clamp_max(1, -1));
        assert_eq!(-2, clamp_max(-2, -1));

        // Floating tests
        assert_eq!(1.0, clamp(1.0, -1.0, 2.0));
        assert_eq!(-1.0, clamp(-2.0, -1.0, 2.0));
        assert_eq!(2.0, clamp(3.0, -1.0, 2.0));
        assert_eq!(1.0, clamp_min(1.0, -1.0));
        assert_eq!(-1.0, clamp_min(-2.0, -1.0));
        assert_eq!(-1.0, clamp_max(1.0, -1.0));
        assert_eq!(-2.0, clamp_max(-2.0, -1.0));
        assert!(clamp(f32::NAN, -1.0, 1.0).is_nan());
        assert!(clamp_min(f32::NAN, 1.0).is_nan());
        assert!(clamp_max(f32::NAN, 1.0).is_nan());
    }
}
