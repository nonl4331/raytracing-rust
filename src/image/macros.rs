//------
// MISC
//------
#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $z:expr) => {
        ultraviolet::Vec3::new($x as f32, $y as f32, $z as f32)
    };
    ($x:expr, $y:expr) => {
        ultraviolet::Vec2::new($x as f32, $y as f32)
    };
}

#[macro_export]
macro_rules! colour {
    ($r:expr,$g:expr,$b:expr) => {
        crate::ray_tracing::ray::Colour::new($r as f32, $g as f32, $b as f32)
    };
    ($value:expr) => {
        crate::ray_tracing::ray::Colour::new($value as f32, $value as f32, $value as f32)
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
        crate::ray_tracing::texture::Texture::Lerp(crate::ray_tracing::texture::Lerp::new(
            $colour_one,
            $colour_two,
        ))
    };
    ($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
        crate::ray_tracing::texture::Texture::Lerp(crate::ray_tracing::texture::Lerp::new(
            colour!($r1, $g1, $b1),
            colour!($r2, $g2, $b2),
        ))
    };
}

//-----
// MATERIALS
//-----
#[macro_export]
macro_rules! diffuse {
    ($r:expr,$g:expr,$b:expr, $absorption:expr) => {
        crate::ray_tracing::material::Material::Diffuse(
            crate::ray_tracing::material::Diffuse::new(
                &std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $absorption as f32,
            ),
        );
    };
    ($texture:expr,$absorption:expr) => {
        crate::ray_tracing::material::Material::Diffuse(
            crate::ray_tracing::material::Diffuse::new($texture, $absorption as f32),
        );
    };
}

#[macro_export]
macro_rules! reflect {
    ($r:expr,$g:expr,$b:expr, $fuzz:expr) => {
        crate::ray_tracing::material::Material::Reflect(
            crate::ray_tracing::material::Reflect::new(
                &Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $fuzz as f32,
            ),
        );
    };
    ($texture:expr,$fuzz:expr) => {
        crate::ray_tracing::material::Material::Reflect(
            crate::ray_tracing::material::Reflect::new($texture, $fuzz as f32),
        );
    };
}

#[macro_export]
macro_rules! refract {
    ($r:expr,$g:expr,$b:expr, $eta:expr) => {
        crate::ray_tracing::material::Material::Refract(
            crate::ray_tracing::material::Refract::new(
                &std::sync::Arc::new(crate::ray_tracing::texture::Texture::SolidColour(
                    crate::ray_tracing::texture::SolidColour::new(colour!($r, $g, $b)),
                )),
                $eta as f32,
            ),
        );
    };
    ($texture:expr,$eta:expr) => {
        crate::ray_tracing::material::Material::Refract(
            crate::ray_tracing::material::Refract::new($texture, $eta as f32),
        );
    };
}

#[macro_export]
macro_rules! emit {
    ($r:expr,$g:expr,$b:expr, $strength:expr) => {
        crate::ray_tracing::material::Material::Emit(crate::ray_tracing::material::Emit::new(
            &std::sync::Arc::new(Texture::SolidColour(SolidColour::new(colour!($r, $g, $b)))),
            $strength as f32,
        ));
    };
    ($texture:expr,$strength:expr) => {
        crate::ray_tracing::material::Material::Emit(crate::ray_tracing::material::Emit::new(
            $texture,
            $strength as f32,
        ));
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
                $radius as f32,
                $material,
            ),
        )
    };
    ($position:expr, $radius:expr, $material:expr) => {
        crate::ray_tracing::primitives::Primitive::Sphere(
            crate::ray_tracing::primitives::Sphere::new($position, $radius as f32, $material),
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
                $axis_value as f32,
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
            $fov as f32,
            $aspect_ratio as f32,
            $aperture as f32,
            $focus_dist as f32,
            $sky,
            $split_type,
            $primitives,
        )
    };
}
