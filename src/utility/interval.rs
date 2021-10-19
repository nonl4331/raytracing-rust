use crate::utility::math::{next_float, previous_float, Float};
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(all(feature = "f64"))]
use std::f64::{INFINITY, NEG_INFINITY};

#[cfg(not(feature = "f64"))]
use std::f32::{INFINITY, NEG_INFINITY};

#[derive(Debug, Copy, Clone)]
pub struct Interval {
    min: Float,
    max: Float,
}

impl Interval {
    pub fn new(min: Float, max: Float) -> Self {
        Interval { min, max }
    }
    pub fn from_float(val: Float) -> Self {
        Interval::new(val, val)
    }
    pub fn low(self) -> Float {
        self.min
    }
    pub fn high(self) -> Float {
        self.max
    }
    pub fn average(self) -> Float {
        (self.min + self.max) / 2.0
    }
    pub fn sqrt(self) -> Self {
        Interval::new(previous_float(self.min.sqrt()), next_float(self.max.sqrt()))
    }
    pub fn signum(self) -> Self {
        let val = self.average().signum();
        Interval::new(val, val)
    }
    pub fn abs(self) -> Self {
        let min = self.min.abs();
        let max = self.max.abs();
        if min > max {
            Interval::new(max, min)
        } else {
            Interval::new(min, max)
        }
    }
    pub fn contains(&self, val: Float) -> bool {
        val < self.max && val > self.min
    }
}

impl Add for Interval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Interval::new(
            previous_float(self.min + rhs.min),
            next_float(self.max + rhs.max),
        )
    }
}

impl AddAssign for Interval {
    fn add_assign(&mut self, rhs: Self) {
        *self = Interval::new(
            previous_float(self.min + rhs.min),
            next_float(self.max + rhs.max),
        );
    }
}

impl Sub for Interval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Interval::new(
            previous_float(self.min - rhs.max),
            next_float(self.max - rhs.min),
        )
    }
}

impl SubAssign for Interval {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Interval::new(
            previous_float(self.min - rhs.max),
            next_float(self.max - rhs.min),
        );
    }
}

impl Mul for Interval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let res = [
            self.min * rhs.min,
            self.min * rhs.max,
            self.max * rhs.min,
            self.max * rhs.max,
        ];
        let max = res[1..3]
            .iter()
            .fold(next_float(res[0]), |max, &val| max.max(next_float(val)));
        let min = res[1..3].iter().fold(previous_float(res[0]), |min, &val| {
            min.min(previous_float(val))
        });
        Interval::new(min, max)
    }
}

impl MulAssign for Interval {
    fn mul_assign(&mut self, rhs: Self) {
        let res = [
            self.min * rhs.min,
            self.min * rhs.max,
            self.max * rhs.min,
            self.max * rhs.max,
        ];
        let max = res[1..3]
            .iter()
            .fold(next_float(res[0]), |max, &val| max.max(next_float(val)));
        let min = res[1..3].iter().fold(previous_float(res[0]), |min, &val| {
            min.min(previous_float(val))
        });
        *self = Interval::new(min, max);
    }
}

impl Div for Interval {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.contains(0.0) {
            return Interval::new(NEG_INFINITY, INFINITY);
        }

        let res = [
            self.min / rhs.min,
            self.min / rhs.max,
            self.max / rhs.min,
            self.max / rhs.max,
        ];

        let max = res[1..3]
            .iter()
            .fold(next_float(res[0]), |max, &val| max.max(next_float(val)));
        let min = res[1..3].iter().fold(previous_float(res[0]), |min, &val| {
            min.min(previous_float(val))
        });
        Interval::new(min, max)
    }
}

impl DivAssign for Interval {
    fn div_assign(&mut self, rhs: Self) {
        if rhs.contains(0.0) {
            *self = Interval::new(NEG_INFINITY, INFINITY);
        }

        let res = [
            self.min / rhs.min,
            self.min / rhs.max,
            self.max / rhs.min,
            self.max / rhs.max,
        ];

        let max = res[1..3]
            .iter()
            .fold(next_float(res[0]), |max, &val| max.max(next_float(val)));
        let min = res[1..3].iter().fold(previous_float(res[0]), |min, &val| {
            min.min(previous_float(val))
        });
        *self = Interval::new(min, max);
    }
}

impl Neg for Interval {
    type Output = Self;
    fn neg(self) -> Self {
        Interval::new(-self.max, -self.min)
    }
}

impl PartialEq for Interval {
    fn eq(&self, rhs: &Self) -> bool {
        self.min == rhs.min && self.max == rhs.max
    }
    fn ne(&self, rhs: &Self) -> bool {
        self.min != rhs.min || self.max != rhs.max
    }
}

impl PartialEq<Float> for Interval {
    fn eq(&self, rhs: &Float) -> bool {
        rhs >= &self.min && rhs <= &self.max
    }
    fn ne(&self, rhs: &Float) -> bool {
        rhs < &self.min || rhs > &self.max
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        (self.min + self.max).partial_cmp(&(rhs.min + rhs.max))
    }
}

impl PartialOrd<Float> for Interval {
    fn partial_cmp(&self, rhs: &Float) -> Option<Ordering> {
        (self.min + self.max).partial_cmp(rhs)
    }
}
