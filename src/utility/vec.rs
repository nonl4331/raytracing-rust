use crate::math::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! expr {
    ($e:expr) => {
        $e
    };
}

macro_rules! impl_operator {
    ($name:ident, $function_name:ident, $operator:tt) => {
        impl $name for Vec3 {
        	type Output = Self;
            #[inline]
        	fn $function_name(self, rhs: Self) -> Self {
        		Vec3::new(expr!(self.x $operator rhs.x), expr!(self.y $operator rhs.y), expr!(self.z $operator rhs.z))
        	}
        }
    };
}

macro_rules! impl_operator_float {
    ($name:ident, $function_name:ident, $operator:tt) => {
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
        impl $name<Float> for Vec3 {
            fn $function_name(&mut self, rhs: Float) {
                expr!(self.x $operator rhs);
                expr!(self.y $operator rhs);
                expr!(self.z $operator rhs);
            }
        }
    };
}

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Vec3 {
    #[inline]
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        Vec3 { x, y, z }
    }

    #[inline]
    pub fn from_uv(vec: ultraviolet::Vec3) -> Self {
        Vec3::new(vec.x, vec.y, vec.z)
    }

    #[inline]
    pub fn to_uv(self) -> ultraviolet::Vec3 {
        ultraviolet::Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn dot(&self, other: Self) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
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
}

impl_operator!(Add, add, +);
impl_operator_float!(Add, add, +);
impl_operator_float_assign!(AddAssign, add_assign, +=);
impl_operator!(Sub, sub, -);
impl_operator_float!(Sub, sub, -);
impl_operator_float_assign!(SubAssign, sub_assign, -=);
impl_operator!(Mul, mul, *);
impl_operator_float!(Mul, mul, *);
impl_operator_float_assign!(MulAssign, mul_assign, *=);
impl_operator!(Div, div, /);
impl_operator_float!(Div, div, /);
impl_operator_float_assign!(DivAssign, div_assign, /=);

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}
