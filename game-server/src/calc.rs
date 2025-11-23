use std::ops::{Add, Deref, Div, Mul};

#[derive(Debug, Clone, Copy)]
pub struct BasisPoint<T> {
    value: T,
}

#[derive(Debug, Clone, Copy)]
pub struct ModifierInstance<T> {
    id: i64,
    modifier: Modifier<T>,
    priority: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Modifier<T> {
    Add(T),
    Multiply(BasisPoint<T>),
    Set(T),
}

pub trait Modifiable<T>
{
    fn add_modifier(&mut self, modifier: ModifierInstance<T>);
    fn remove_modifier(&mut self, id: i64);
    fn recalculate(&mut self);
}

#[derive(Debug)]
pub struct BasedValue<T>
{
    value: T,
    base: T,
    modifiers: Vec<ModifierInstance<T>>, //TODO: Consider using `smallvec` crate if optimization is needed.
}

#[derive(Debug)]
pub struct BasedRange<T>
{
    inner: BasedValue<T>,
    min: T,
    max: T,
}

impl Mul<BasisPoint<i64>> for i64 {
    type Output = i64;

    fn mul(self, rhs: BasisPoint<i64>) -> Self::Output {
        (self * rhs.value) / 10000
    }
}

impl<T> ModifierInstance<T> {
    pub fn new(id: i64, modifier: Modifier<T>, priority: u8,) -> Self {
        Self { id, modifier, priority }
    }
}

impl<T> BasedValue<T>
where
    T: Copy,
{
    pub fn new(base: T) -> BasedValue<T> {
        BasedValue {
            base,
            value: base,
            modifiers: Vec::new(),
        }
    }
}

impl<T> Modifiable<T> for BasedValue<T>
where
    T: Copy + Add<Output = T> + Mul<BasisPoint<T>, Output = T>,
{
    fn add_modifier(&mut self, modifier: ModifierInstance<T>) {
        self.modifiers.push(modifier);
        self.recalculate();
    }

    fn remove_modifier(&mut self, id: i64) {
        if let Some(index) = self.modifiers.iter().position(|m| m.id == id) {
            self.modifiers.swap_remove(index);
            self.recalculate();
        }
    }

    fn recalculate(&mut self) {
        self.modifiers.sort_by_key(|m| std::cmp::Reverse(m.priority));

        self.value = self.base;

        for modifier in &self.modifiers {
            match modifier.modifier {
                Modifier::Add(amount) => {
                    self.value = self.value + amount;
                }
                Modifier::Multiply(factor) => {
                    self.value = self.value * factor;
                }
                Modifier::Set(new_value) => {
                    self.value = new_value;
                }
            }
        }
    }
}

impl<T> Deref for BasedValue<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Deref for BasedRange<T>
{
    type Target = BasedValue<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> BasedRange<T>
where
    T: Copy,
{
    pub fn new(base: T, min: T, max: T) -> BasedRange<T> {
        BasedRange {
            inner: BasedValue::new(base),
            min,
            max,
        }
    }
}

impl<T> Modifiable<T> for BasedRange<T>
where
    T: Copy + PartialOrd + Add<Output = T> + Mul<BasisPoint<T>, Output = T>,
{
    fn add_modifier(&mut self, modifier: ModifierInstance<T>) {
        self.inner.add_modifier(modifier);
        self.inner.value = clamp(self.inner.value, self.min, self.max);
    }

    fn remove_modifier(&mut self, id: i64) {
        self.inner.remove_modifier(id);
        self.inner.value = clamp(self.inner.value, self.min, self.max);
    }

    fn recalculate(&mut self) {
        self.inner.recalculate();
        self.inner.value = clamp(self.inner.value, self.min, self.max);
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
