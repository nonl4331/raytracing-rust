use crate::math::{gamma, next_float, previous_float};

use crate::ray_tracing::{
    primitives::Sphere,
    ray::Ray,
    tracing::{Hit, PrimitiveTrait},
};

const SPHERE_INTERSECTION: SphereIntersection = if cfg!(sphere_three) {
    SphereIntersection::Three
} else if cfg!(sphere_two) {
    SphereIntersection::Two
} else {
    SphereIntersection::One
};

enum SphereIntersection {
    One,
    Two,
    Three,
}

use crate::utility::vec::Vec3;

/*pub fn test_offset_ray(origin: IntervalVec3, normal: Vec3, is_brdf: bool) -> Vec3 {
    let offset_val = normal.abs().dot(10.0 * origin.error());
    let mut offset = offset_val * normal;

    if !is_brdf {
        offset = -offset;
    }

    let mut new_origin = origin.average() + offset;

    if offset.x > 0.0 {
        new_origin.x = next_float(new_origin.x);
    } else {
        new_origin.x = previous_float(new_origin.x);
    }

    if offset.y > 0.0 {
        new_origin.y = next_float(new_origin.y);
    } else {
        new_origin.y = previous_float(new_origin.y);
    }

    if offset.z > 0.0 {
        new_origin.z = next_float(new_origin.z);
    } else {
        new_origin.z = previous_float(new_origin.z);
    }

    new_origin
}*/

pub fn offset_ray(origin: Vec3, normal: Vec3, error: Vec3, is_brdf: bool) -> Vec3 {
    let offset_val = normal.abs().dot(error);
    let mut offset = offset_val * normal;

    if !is_brdf {
        offset = -offset;
    }

    let mut new_origin = origin + offset;

    if offset.x > 0.0 {
        new_origin.x = next_float(new_origin.x);
    } else {
        new_origin.x = previous_float(new_origin.x);
    }

    if offset.y > 0.0 {
        new_origin.y = next_float(new_origin.y);
    } else {
        new_origin.y = previous_float(new_origin.y);
    }

    if offset.z > 0.0 {
        new_origin.z = next_float(new_origin.z);
    } else {
        new_origin.z = previous_float(new_origin.z);
    }

    new_origin
}

pub fn sphere_intersection(sphere: &Sphere, ray: &Ray, is_brdf: bool) -> Option<Hit> {
    match SPHERE_INTERSECTION {
        SphereIntersection::One => sphere_intersection_one(sphere, ray, is_brdf),
        SphereIntersection::Two => sphere_intersection_two(sphere, ray, is_brdf),
        SphereIntersection::Three => sphere_intersection_three(sphere, ray, is_brdf),
    }
}

// baseline algorithm (exibits some issues)
pub fn sphere_intersection_one(sphere: &Sphere, ray: &Ray, is_brdf: bool) -> Option<Hit> {
    let oc = ray.origin - sphere.center;
    let a = ray.direction.dot(ray.direction);
    let h = ray.direction.dot(oc); // b / 2 ( becuase of slightly simplified quadratic formula)
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
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
        let mut point = ray.at(t);
        let mut normal = (point - sphere.center) / sphere.radius;
        let mut out = true;

        // make sure normal is opposite ray direction and check if ray is inside object
        if normal.dot(ray.direction) > 0.0 {
            normal = -normal;
            out = false;
        }
        // offset point along normal
        if is_brdf {
            point = point + 0.0001 * normal;
        } else {
            point = point - 0.0001 * normal;
        }

        // return hit
        Some(Hit {
            t,
            point: point,
            normal: normal,
            uv: sphere.get_uv(point),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}

pub fn sphere_intersection_two(sphere: &Sphere, ray: &Ray, is_brdf: bool) -> Option<Hit> {
    let center = Vec3::from_uv(sphere.center);
    let radius = sphere.radius;
    let direction = Vec3::from_uv(ray.direction);
    let origin = Vec3::from_uv(ray.origin);

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
        let point = offset_ray(point, normal, point_error, is_brdf) + center;
        Some(Hit {
            t,
            point: point.to_uv(),
            normal: normal.to_uv(),
            uv: sphere.get_uv(point.to_uv()),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}

pub fn sphere_intersection_three(sphere: &Sphere, ray: &Ray, is_brdf: bool) -> Option<Hit> {
    let dir = Vec3::from_uv(ray.direction);
    let center = Vec3::from_uv(sphere.center);
    let radius = sphere.radius;
    let orig = Vec3::from_uv(ray.origin);

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
        let point = Vec3::from_uv(ray.at(t));

        // Get normal from intersection point
        let mut normal = (point - center) / radius;

        // Make sure normal faces outward and make note of what side of the object the ray is on
        let mut out = true;
        if normal.dot(dir) > 0.0 {
            out = false;
            normal = -normal;
        }

        // offset point along normal
        let point = offset_ray(point, normal, 0.000001 * Vec3::new(1.0, 1.0, 1.0), is_brdf);

        // fill in details about intersection point
        Some(Hit {
            t,
            point: point.to_uv(),
            normal: normal.to_uv(),
            uv: sphere.get_uv(point.to_uv()),
            out,
            material: sphere.material.clone(),
        })
    } else {
        None
    }
}
