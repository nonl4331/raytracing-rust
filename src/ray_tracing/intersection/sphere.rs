use crate::ray_tracing::{
    intersection::{Hit, PrimitiveTrait},
    primitives::Sphere,
    ray::Ray,
};
use crate::utility::{interval::Interval, interval_vec::IntervalVec3, math::gamma, vec::Vec3};

const SPHERE_INTERSECTION: SphereIntersection = if cfg!(feature = "sphere_three") {
    SphereIntersection::Three
} else if cfg!(feature = "sphere_two") {
    SphereIntersection::Two
} else {
    SphereIntersection::One
};

enum SphereIntersection {
    One,
    Two,
    Three,
}

pub fn sphere_intersection(sphere: &Sphere, ray: &Ray) -> Option<Hit> {
    match SPHERE_INTERSECTION {
        SphereIntersection::One => sphere_intersection_one(sphere, ray),
        SphereIntersection::Two => sphere_intersection_two(sphere, ray),
        SphereIntersection::Three => sphere_intersection_three(sphere, ray),
    }
}

// baseline algorithm
pub fn sphere_intersection_one(sphere: &Sphere, ray: &Ray) -> Option<Hit> {
    let origin = IntervalVec3::from_vec(ray.origin);
    let center = IntervalVec3::from_vec(sphere.center);
    let direction = IntervalVec3::from_vec(ray.direction);
    let radius = Interval::from_float(sphere.radius);

    let oc = origin - center;
    let a = direction.dot(direction);
    let h = direction.dot(oc); // b / 2 ( becuase of slightly simplified quadratic formula)
    let c = oc.dot(oc) - radius * radius;
    let disc = h * h - a * c;

    if disc > 0.0 {
        // get smaller solution
        let mut t = (-h - disc.sqrt()) / a;

        // if smaller solution is less than zero use other solution
        // if its less than zero it will get handled outside of this function
        if t < 0.0 {
            t = (-h + disc.sqrt()) / a;
        }

        // get point, normal (opposite ray direction) and check if ray is inside object
        let point = origin + direction * t;
        let mut normal = (point - center) / radius;
        let mut out = true;

        // make sure normal is opposite ray direction and check if ray is inside object
        if normal.dot(direction) > 0.0 {
            normal = -normal;
            out = false;
        }

        // return hit
        Some(Hit {
            t: t.average(),
            point: point.average(),
            error: point.error(),
            normal: normal.average(),
            uv: sphere.get_uv(point.average()),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}

pub fn sphere_intersection_two(sphere: &Sphere, ray: &Ray) -> Option<Hit> {
    let center = sphere.center;
    let radius = sphere.radius;
    let direction = ray.direction;
    let origin = ray.origin;

    let oc = origin - center;

    let b_prime = -oc.dot(direction);

    let disc = radius * radius - (oc + b_prime * direction).mag_sq();

    let c = oc.dot(oc) - radius * radius;

    if disc > 0.0 {
        let q = b_prime + b_prime.signum() * disc.sqrt();
        let mut t0 = q;
        let mut t1 = c / q;
        if t1 < t0 {
            std::mem::swap(&mut t0, &mut t1)
        }
        let t = if t1 < 0.0 {
            return None;
        } else if t0 < 0.0 {
            t1
        } else {
            t0
        };

        let mut point = oc + direction * t;
        point *= radius / point.mag();
        let mut normal = point / radius;
        point = point + center;
        let mut out = true;
        if normal.dot(direction) > 0.0 {
            normal *= -1.0;
            out = false;
        }
        let point_error = gamma(5) * point.abs();

        let point = point + center;

        Some(Hit {
            t,
            point,
            error: point_error,
            normal,
            uv: sphere.get_uv(point),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}

pub fn sphere_intersection_three(sphere: &Sphere, ray: &Ray) -> Option<Hit> {
    let dir = ray.direction;
    let center = sphere.center;
    let radius = sphere.radius;
    let orig = ray.origin;

    // simplified terms for algorithm below
    let deltap = center - orig;
    let ddp = dir.dot(deltap);
    let deltapdot = deltap.dot(deltap);

    let remedy_term = deltap - ddp * dir;
    let discriminant = radius * radius - remedy_term.dot(remedy_term);

    // check if any solutions exist
    if discriminant > 0.0 {
        // the square root of the discriminant
        let sqrt_val = discriminant.sqrt();

        // Get intermediate q value based on ddp sign
        let q = if ddp > 0.0 {
            ddp + sqrt_val
        } else {
            ddp - sqrt_val
        };

        // Get two solutions of quadratic formula
        let mut t0 = q;
        let mut t1 = (deltapdot - radius * radius) / q;

        // Make sure t1 > t0 (for sorting purposes)
        if t1 < t0 {
            std::mem::swap(&mut t0, &mut t1);
        };

        // Get smallest t value that is above 0
        let t = if t0 > 0.0 {
            t0
        } else {
            if t1 > 0.0 {
                return None;
            }
            t0
        };

        // Get point at "t"
        let point = ray.at(t);

        // Get normal from intersection point
        let mut normal = (point - center) / radius;

        // Make sure normal faces outward and make note of what side of the object the ray is on
        let mut out = true;
        if normal.dot(dir) > 0.0 {
            out = false;
            normal = -normal;
        }

        // fill in details about intersection point
        Some(Hit {
            t,
            point,
            error: 0.000001 * Vec3::one(),
            normal,
            uv: sphere.get_uv(point),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}
