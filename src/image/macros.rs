//------
// MISC
//------
#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $z:expr) => {
        crate::utility::vec::Vec3::new(
            $x as crate::utility::math::Float,
            $y as crate::utility::math::Float,
            $z as crate::utility::math::Float,
        )
    };
    ($x:expr, $y:expr) => {
        crate::utility::vec::Vec2::new(
            $x as crate::utility::math::Float,
            $y as crate::utility::math::Float,
        )
    };
}

#[macro_export]
macro_rules! colour {
    ($r:expr,$g:expr,$b:expr) => {
        crate::ray_tracing::ray::Colour::new(
            $r as crate::utility::math::Float,
            $g as crate::utility::math::Float,
            $b as crate::utility::math::Float,
        )
    };
    ($value:expr) => {
        crate::ray_tracing::ray::Colour::new(
            $value as crate::utility::math::Float,
            $value as crate::utility::math::Float,
            $value as crate::utility::math::Float,
        )
    };
}

#[macro_export]
macro_rules! axis {
    (X) => {
        crate::ray_tracing::primitives::Axis::X
    };
    (Y) => {
        crate::ray_tracing::primitives::Axis::Y
    };
    (Z) => {
        crate::ray_tracing::primitives::Axis::Z
    };
}

#[macro_export]
macro_rules! partition {
    ($array:expr, $closure:expr) => {{
        let len = $array.len();
        let (mut left, mut right) = (0, len - 1);
        let mid_index: usize;

        loop {
            while left < len && $closure(&$array[left]) {
                left += 1;
            }

            while right > 0 && !($closure(&$array[right])) {
                right -= 1;
            }

            if left >= right {
                mid_index = left;
                break;
            }
            $array.swap(left, right);
        }
        mid_index
    }};
}

//-----
// TEXTURES
//-----
#[macro_export]
macro_rules! solid_colour {
    ($r:expr, $g:expr, $b:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
            crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
        ))
    };
    ($colour:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
            crate::ray_tracing::texture::SolidColour::new($colour),
        ))
    };
}

#[macro_export]
macro_rules! image {
    ($filepath:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::ImageTexture(
            crate::ray_tracing::texture::ImageTexture::new($filepath),
        ))
    };
}

#[macro_export]
macro_rules! checkered {
    ($colour_one:expr, $colour_two:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::CheckeredTexture(
            crate::ray_tracing::texture::CheckeredTexture::new($colour_one, $colour_two),
        ))
    };
    ($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::CheckeredTexture(
            crate::ray_tracing::texture::CheckeredTexture::new(
                colour!($r1, $g1, $b1),
                colour!($r2, $g2, $b2),
            ),
        ))
    };
}

#[macro_export]
macro_rules! texture_lerp {
    ($colour_one:expr, $colour_two:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::Lerp(
            crate::ray_tracing::texture::Lerp::new($colour_one, $colour_two),
        ))
    };
    ($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::Lerp(
            crate::ray_tracing::texture::Lerp::new(colour!($r1, $g1, $b1), colour!($r2, $g2, $b2)),
        ))
    };
}

#[macro_export]
macro_rules! perlin {
    () => {
        std::sync::Arc::new(crate::ray_tracing::texture::Texture::Perlin(Box::new(
            crate::ray_tracing::texture::Perlin::new(),
        )))
    };
}

//-----
// MATERIALS
//-----
#[macro_export]
macro_rules! diffuse {
    ($r:expr,$g:expr,$b:expr, $absorption:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Lambertian(
            crate::ray_tracing::material::Lambertian::new(
                &std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $absorption as crate::utility::math::Float,
            ),
        ))
    };
    ($texture:expr,$absorption:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Lambertian(
            crate::ray_tracing::material::Lambertian::new(
                $texture,
                $absorption as crate::utility::math::Float,
            ),
        ))
    };
}

#[macro_export]
macro_rules! reflect {
    ($r:expr,$g:expr,$b:expr, $fuzz:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Reflect(
            crate::ray_tracing::material::Reflect::new(
                &Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $fuzz as crate::utility::math::Float,
            ),
        ));
    };
    ($texture:expr,$fuzz:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Reflect(
            crate::ray_tracing::material::Reflect::new(
                $texture,
                $fuzz as crate::utility::math::Float,
            ),
        ))
    };
}

#[macro_export]
macro_rules! refract {
    ($r:expr,$g:expr,$b:expr, $eta:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Refract(
            crate::ray_tracing::material::Refract::new(
                &std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $eta as crate::utility::math::Float,
            ),
        ))
    };
    ($texture:expr,$eta:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Refract(
            crate::ray_tracing::material::Refract::new(
                $texture,
                $eta as crate::utility::math::Float,
            ),
        ))
    };
}

#[macro_export]
macro_rules! emit {
    ($r:expr,$g:expr,$b:expr, $strength:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Emit(
            crate::ray_tracing::material::Emit::new(
                &std::sync::Arc::new(Texture::SolidColour(SolidColour::new(colour!($r, $g, $b)))),
                $strength as crate::utility::math::Float,
            ),
        ));
    };
    ($texture:expr,$strength:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::Emit(
            crate::ray_tracing::material::Emit::new(
                $texture,
                $strength as crate::utility::math::Float,
            ),
        ))
    };
}

#[macro_export]
macro_rules! cook_torrence {
    ($r:expr,$g:expr,$b:expr, $alpha:expr, $absorption:expr, $specular_chance:expr, $f0:expr) => {
        std::sync::Arc::new(crate::ray_tracing::material::Material::CookTorrence(
            crate::ray_tracing::material::CookTorrence::new(
                &std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $alpha as crate::utility::math::Float,
                $absorption as crate::utility::math::Float,
                $specular_chance as crate::utility::math::Float,
                $f0,
            ),
        ))
    };
}

//-----
// PRIMITIVES
//-----
#[macro_export]
macro_rules! sphere {
    ($x:expr, $y:expr, $z:expr, $radius:expr, $material:expr) => {
        crate::ray_tracing::primitives::Primitive::Sphere(
            crate::ray_tracing::primitives::Sphere::new(
                position!($x, $y, $z),
                $radius as Float,
                $material,
            ),
        )
    };
    ($position:expr, $radius:expr, $material:expr) => {
        crate::ray_tracing::primitives::Primitive::Sphere(
            crate::ray_tracing::primitives::Sphere::new(
                $position,
                $radius as crate::utility::math::Float,
                $material,
            ),
        )
    };
}

#[macro_export]
macro_rules! aarect {
    ($point_one:expr, $point_two:expr, $axis_value:expr, $axis:expr,  $material:expr) => {
        crate::ray_tracing::primitives::Primitive::AARect(
            crate::ray_tracing::primitives::AARect::new(
                $point_one,
                $point_two,
                $axis_value as crate::utility::math::Float,
                $axis,
                $material,
            ),
        )
    };
    ($x1:expr, $y1:expr, $x2:expr, $y2:expr, $axis_value:expr, $axis:expr,  $material:expr) => {
        crate::ray_tracing::primitives::Primitive::AARect(
            crate::ray_tracing::primitives::AARect::new(
                position!($x1, $y1),
                position!($x2, $y2),
                $axis_value,
                $axis,
                $material,
            ),
        )
    };
}

#[macro_export]
macro_rules! aacuboid {
    ($point_one:expr, $point_two:expr, $material:expr) => {
        crate::ray_tracing::primitives::Primitive::AACubiod(
            crate::ray_tracing::primitives::AACuboid::new($point_one, $point_two, $material),
        )
    };
    ($x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $material:expr) => {
        crate::ray_tracing::primitives::Primitive::AACuboid(
            crate::ray_tracing::primitives::AACuboid::new(
                position!($x1, $y1, $z1),
                position!($x2, $y2, $z2),
                $material,
            ),
        )
    };
}

#[macro_export]
macro_rules! model {
    ($filepath:expr, $material:expr) => {
        crate::ray_tracing::load_model::load_model($filepath, $material)
    };
}

// assumes orientation
#[macro_export]
macro_rules! triangle {
    ($point_one:expr, $point_two:expr, $point_three:expr, $material:expr) => {{
        let normal = {
            let a = $point_two - $point_one;
            let b = $point_three - $point_one;
            a.cross(b)
        }
        .normalized();

        crate::ray_tracing::primitives::Primitive::Triangle(
            crate::ray_tracing::primitives::Triangle::new_from_arc(
                [$point_one, $point_two, $point_two],
                [normal; 3],
                $material,
            ),
        )
    }};

    ($p1x:expr, $p1y:expr, $p1z:expr, $p2x:expr, $p2y:expr, $p2z:expr, $p3x:expr, $p3y:expr, $p3z:expr, $material:expr) => {{
        let point_one = position!($p1x, $p1y, $p1z);
        let point_two = position!($p2x, $p2y, $p2z);
        let point_three = position!($p3x, $p3y, $p3z);
        let normal = {
            let a = point_two - point_one;
            let b = point_three - point_one;
            a.cross(b)
        }
        .normalized();

        crate::ray_tracing::primitives::Primitive::Triangle(
            crate::ray_tracing::primitives::Triangle::new_from_arc(
                [point_one, point_two, point_two],
                [normal; 3],
                $material,
            ),
        )
    }};
}

//-----
// OTHER STRUCTURES
//-----
#[macro_export]
macro_rules! sky {
    () => {
        crate::ray_tracing::sky::Sky::new(None)
    };
    ($sky_texture:expr) => {
        crate::ray_tracing::sky::Sky::new(Some($sky_texture))
    };
}

#[macro_export]
macro_rules! scene {
    ($origin:expr, $lookat:expr, $vup:expr, $fov:expr, $aspect_ratio:expr, $aperture:expr, $focus_dist:expr, $sky:expr, $split_type:expr, $primitives:expr) => {
        crate::image::scene::Scene::new(
            $origin,
            $lookat,
            $vup,
            $fov as crate::utility::math::Float,
            $aspect_ratio as crate::utility::math::Float,
            $aperture as crate::utility::math::Float,
            $focus_dist as crate::utility::math::Float,
            $sky,
            $split_type,
            $primitives,
        )
    };
}
