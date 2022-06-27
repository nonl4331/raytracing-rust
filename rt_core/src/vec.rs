use crate::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! expr {
	($e:expr) => {
		$e
	};
}

macro_rules! impl_operator {
    ($name:ident, $function_name:ident, $operator:tt) => {
        // Vec2
        impl $name for Vec2 {
        	type Output = Self;
            #[inline]
        	fn $function_name(self, rhs: Self) -> Self {
        		Vec2::new(expr!(self.x $operator rhs.x), expr!(self.y $operator rhs.y))
        	}
        }
        // Vec3
        impl $name for Vec3 {
            type Output = Self;
            #[inline]
            fn $function_name(self, rhs: Self) -> Self {
                Vec3::new(expr!(self.x $operator rhs.x), expr!(self.y $operator rhs.y), expr!(self.z $operator rhs.z))
            }
        }
    };
}

macro_rules! impl_operator_assign {
    ($name:ident, $function_name:ident, $operator:tt) => {
        // Vec2
        impl $name for Vec2 {
            #[inline]
            fn $function_name(&mut self, rhs: Self) {
                expr!(self.x $operator rhs.x);
                expr!(self.y $operator rhs.y);
            }
        }
        // Vec3
        impl $name for Vec3 {
            #[inline]
            fn $function_name(&mut self, rhs: Self) {
                expr!(self.x $operator rhs.x);
                expr!(self.y $operator rhs.y);
                expr!(self.z $operator rhs.z);
            }
        }
    };
}

macro_rules! impl_operator_float {
    ($name:ident, $function_name:ident, $operator:tt) => {
        // Vec2
        impl $name<Float> for Vec2 {
            type Output = Self;
            #[inline]
            fn $function_name(self, rhs: Float) -> Self {
                Vec2::new(expr!(self.x $operator rhs), expr!(self.y $operator rhs))
            }
        }
        impl $name<Vec2> for Float {
            type Output = Vec2;
            #[inline]
            fn $function_name(self, rhs: Vec2) -> Vec2 {
                Vec2::new(expr!(self $operator rhs.x), expr!(self $operator rhs.y))
            }
        }
        // Vec3
        impl $name<Float> for Vec3 {
            type Output = Self;
            #[inline]
            fn $function_name(self, rhs: Float) -> Self {
                Vec3::new(expr!(self.x $operator rhs), expr!(self.y $operator rhs), expr!(self.z $operator rhs))
            }
        }
        impl $name<Vec3> for Float {
            type Output = Vec3;
            #[inline]
            fn $function_name(self, rhs: Vec3) -> Vec3 {
                Vec3::new(expr!(self $operator rhs.x), expr!(self $operator rhs.y), expr!(self $operator rhs.z))
            }
        }
    };
}

macro_rules! impl_operator_float_assign {
    ($name:ident, $function_name:ident, $operator:tt) => {
        // Vec2
        impl $name<Float> for Vec2 {
            fn $function_name(&mut self, rhs: Float) {
                expr!(self.x $operator rhs);
                expr!(self.y $operator rhs);
            }
        }
        // Vec3
        impl $name<Float> for Vec3 {
            fn $function_name(&mut self, rhs: Float) {
                expr!(self.x $operator rhs);
                expr!(self.y $operator rhs);
                expr!(self.z $operator rhs);
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
pub struct Vec3 {
	pub x: Float,
	pub y: Float,
	pub z: Float,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
pub struct Vec2 {
	pub x: Float,
	pub y: Float,
}

impl Vec3 {
	#[inline]
	pub fn new(x: Float, y: Float, z: Float) -> Self {
		Vec3 { x, y, z }
	}

	#[inline]
	pub fn one() -> Self {
		Vec3::new(1.0, 1.0, 1.0)
	}

	#[inline]
	pub fn zero() -> Self {
		Vec3::new(0.0, 0.0, 0.0)
	}

	#[inline]
	pub fn dot(&self, other: Self) -> Float {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	#[inline]
	pub fn cross(&self, other: Self) -> Self {
		Vec3::new(
			self.y * other.z - self.z * other.y,
			self.z * other.x - self.x * other.z,
			self.x * other.y - self.y * other.x,
		)
	}

	#[inline]
	pub fn mag_sq(&self) -> Float {
		self.dot(*self)
	}

	#[inline]
	pub fn mag(&self) -> Float {
		self.dot(*self).sqrt()
	}

	#[inline]
	pub fn normalise(&mut self) {
		*self /= self.mag();
	}

	#[inline]
	pub fn normalised(self) -> Self {
		self / self.mag()
	}
	#[inline]
	pub fn abs(self) -> Self {
		Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
	}
	#[inline]
	pub fn reflect(&mut self, normal: Self) {
		*self -= 2.0 * self.dot(normal) * normal
	}

	#[inline]
	pub fn component_min(self) -> Float {
		self.x.min(self.y.min(self.z))
	}

	#[inline]
	pub fn component_max(self) -> Float {
		self.x.max(self.y.max(self.z))
	}

	#[inline]
	pub fn min_by_component(self, other: Self) -> Self {
		Vec3::new(
			self.x.min(other.x),
			self.y.min(other.y),
			self.z.min(other.z),
		)
	}

	#[inline]
	pub fn max_by_component(self, other: Self) -> Self {
		Vec3::new(
			self.x.max(other.x),
			self.y.max(other.y),
			self.z.max(other.z),
		)
	}

	#[inline]
	pub fn contains_nan(&self) -> bool {
		self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
	}
	#[inline]
	pub fn is_finite(&self) -> bool {
		self.x.is_finite() || self.y.is_finite() || self.z.is_finite()
	}
}

impl Vec2 {
	#[inline]
	pub fn new(x: Float, y: Float) -> Self {
		Vec2 { x, y }
	}

	#[inline]
	pub fn one() -> Self {
		Vec2::new(1.0, 1.0)
	}

	#[inline]
	pub fn zero() -> Self {
		Vec2::new(0.0, 0.0)
	}

	#[inline]
	pub fn dot(&self, other: Self) -> Float {
		self.x * other.x + self.y * other.y
	}

	#[inline]
	pub fn mag_sq(&self) -> Float {
		self.dot(*self)
	}
	#[inline]
	pub fn mag(&self) -> Float {
		self.dot(*self).sqrt()
	}
	#[inline]
	pub fn normalise(&mut self) {
		*self /= self.mag();
	}

	#[inline]
	pub fn normalised(self) -> Self {
		self / self.mag()
	}
	#[inline]
	pub fn abs(self) -> Self {
		Vec2::new(self.x.abs(), self.y.abs())
	}

	#[inline]
	pub fn component_min(self) -> Float {
		self.x.min(self.y)
	}

	#[inline]
	pub fn component_max(self) -> Float {
		self.x.max(self.y)
	}

	#[inline]
	pub fn min_by_component(self, other: Self) -> Self {
		Vec2::new(self.x.min(other.x), self.y.min(other.y))
	}

	#[inline]
	pub fn max_by_component(self, other: Self) -> Self {
		Vec2::new(self.x.max(other.x), self.y.max(other.y))
	}

	#[inline]
	pub fn contains_nan(&self) -> bool {
		self.x.is_nan() || self.y.is_nan()
	}
}

impl_operator!(Add, add, +);
impl_operator_assign!(AddAssign, add_assign, +=);
impl_operator_float!(Add, add, +);
impl_operator_float_assign!(AddAssign, add_assign, +=);

impl_operator!(Sub, sub, -);
impl_operator_assign!(SubAssign, sub_assign, -=);
impl_operator_float!(Sub, sub, -);
impl_operator_float_assign!(SubAssign, sub_assign, -=);

impl_operator!(Mul, mul, *);
impl_operator_assign!(MulAssign, mul_assign, *=);
impl_operator_float!(Mul, mul, *);
impl_operator_float_assign!(MulAssign, mul_assign, *=);

impl_operator!(Div, div, /);
impl_operator_assign!(DivAssign, div_assign, /=);
impl_operator_float!(Div, div, /);
impl_operator_float_assign!(DivAssign, div_assign, /=);

impl Neg for Vec3 {
	type Output = Self;
	#[inline]
	fn neg(self) -> Self {
		Vec3::new(-self.x, -self.y, -self.z)
	}
}

impl std::fmt::Display for Vec3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {}, {})", self.x, self.y, self.z)
	}
}
