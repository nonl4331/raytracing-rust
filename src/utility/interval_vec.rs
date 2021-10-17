use crate::utility::interval::Interval;
use crate::utility::vec::Vec3;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}

macro_rules! impl_operator {
    ($name:ident, $function_name:ident, $operator:tt) => {
        impl $name for IntervalVec3 {
        	type Output = Self;
        	fn $function_name(self, rhs: Self) -> Self {
        		IntervalVec3::new(expr!(self.x $operator rhs.x), expr!(self.y $operator rhs.y), expr!(self.z $operator rhs.z))
        	}
        }
    };
}

macro_rules! impl_operator_interval {
    ($name:ident, $function_name:ident, $operator:tt) => {
        impl $name<Interval> for IntervalVec3 {
            type Output = Self;
            fn $function_name(self, rhs: Interval) -> Self {
                IntervalVec3::new(expr!(self.x $operator rhs), expr!(self.y $operator rhs), expr!(self.z $operator rhs))
            }
        }
        impl $name<IntervalVec3> for Interval {
            type Output = IntervalVec3;
            fn $function_name(self, rhs: IntervalVec3) -> IntervalVec3 {
                IntervalVec3::new(expr!(self $operator rhs.x), expr!(self $operator rhs.y), expr!(self $operator rhs.z))
            }
        }
    };
}

macro_rules! impl_operator_interval_assign {
    ($name:ident, $function_name:ident, $operator:tt) => {
        impl $name<Interval> for IntervalVec3 {
            fn $function_name(&mut self, rhs: Interval) {
                expr!(self.x $operator rhs);
                expr!(self.y $operator rhs);
                expr!(self.z $operator rhs);
            }
        }
    };
}

#[derive(Copy, Clone)]
pub struct IntervalVec3 {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl IntervalVec3 {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        IntervalVec3 { x, y, z }
    }
    pub fn from_uv(vec: crate::utility::vec::Vec3) -> Self {
        IntervalVec3 {
            x: Interval::from_float(vec.x),
            y: Interval::from_float(vec.y),
            z: Interval::from_float(vec.z),
        }
    }
    pub fn dot(&self, other: Self) -> Interval {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn average(self) -> Vec3 {
        0.5 * Vec3::new(
            self.x.low() + self.x.high(),
            self.y.low() + self.y.high(),
            self.z.low() + self.z.high(),
        )
    }
    pub fn mag_sq(self) -> Interval {
        self.dot(self)
    }
    pub fn mag(self) -> Interval {
        self.dot(self).sqrt()
    }
    pub fn normalise(&mut self) {
        *self /= self.mag();
    }
    pub fn normalised(self) -> Self {
        self / self.mag()
    }
    pub fn abs(self) -> Self {
        IntervalVec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
    pub fn error(self) -> Vec3 {
        Vec3::new(
            (self.x.high() - self.x.low()) / 2.0,
            (self.y.high() - self.y.low()) / 2.0,
            (self.z.high() - self.z.low()) / 2.0,
        )
    }
}

impl_operator!(Add, add, +);
impl_operator_interval!(Add, add, +);
impl_operator_interval_assign!(AddAssign, add_assign, +=);
impl_operator!(Sub, sub, -);
impl_operator_interval!(Sub, sub, -);
impl_operator_interval_assign!(SubAssign, sub_assign, -=);
impl_operator!(Mul, mul, *);
impl_operator_interval!(Mul, mul, *);
impl_operator_interval_assign!(MulAssign, mul_assign, *=);
impl_operator!(Div, div, /);
impl_operator_interval!(Div, div, /);
impl_operator_interval_assign!(DivAssign, div_assign, /=);
