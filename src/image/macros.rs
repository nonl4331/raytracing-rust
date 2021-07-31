//------
// MISC
//------
#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $z:expr) => {
        Vec3::new($x as f32, $y as f32, $z as f32)
    };
}

#[macro_export]
macro_rules! colour {
    ($r:expr,$g:expr,$b:expr) => {
        Colour::new($r as f32, $g as f32, $b as f32)
    };
    ($value:expr) => {
        Colour::new($value as f32, $value as f32, $value as f32)
    };
}

//-----
// TEXTURES
//-----
#[macro_export]
macro_rules! solid_colour {
    ($r:expr, $g:expr, $b:expr) => {
        Texture::SolidColour(SolidColour::new(colour!($r, $g, $b)))
    };
    ($colour:expr) => {
        Texture::SolidColour(SolidColour::new($colour))
    };
}

#[macro_export]
macro_rules! image {
    ($filepath:expr) => {
        Texture::ImageTexture(ImageTexture::new($filepath))
    };
}

#[macro_export]
macro_rules! checkered {
    ($colour_one:expr, $colour_two:expr) => {
        Texture::CheckeredTexture(CheckeredTexture::new($colour_one, $colour_two))
    };
    ($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
        Texture::CheckeredTexture(CheckeredTexture::new(
            colour!($r1, $g1, $b1),
            colour!($r2, $g2, $b2),
        ))
    };
}

#[macro_export]
macro_rules! texture_lerp {
    ($colour_one:expr, $colour_two:expr) => {
        Texture::Lerp(Lerp::new($colour_one, $colour_two))
    };
    ($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
        Texture::Lerp(Lerp::new(colour!($r1, $g1, $b1), colour!($r2, $g2, $b2)))
    };
}

//-----
// MATERIALS
//-----
#[macro_export]
macro_rules! diffuse {
    ($r:expr,$g:expr,$b:expr, $absorption:expr) => {
        Material::Diffuse(Diffuse::new(
            Texture::SolidColour(SolidColour::new(colour!($r, $g, $b))),
            $absorption as f32,
        ));
    };
    ($texture:expr,$absorption:expr) => {
        Material::Diffuse(Diffuse::new($texture, $absorption as f32));
    };
}

#[macro_export]
macro_rules! reflect {
    ($r:expr,$g:expr,$b:expr, $fuzz:expr) => {
        Material::Reflect(Reflect::new(
            Texture::SolidColour(SolidColour::new(colour!($r, $g, $b))),
            $fuzz as f32,
        ));
    };
    ($texture:expr,$fuzz:expr) => {
        Material::Reflect(Reflect::new($texture, $fuzz as f32));
    };
}

#[macro_export]
macro_rules! refract {
    ($r:expr,$g:expr,$b:expr, $eta:expr) => {
        Material::Refract(Refract::new(
            Texture::SolidColour(SolidColour::new(colour!($r, $g, $b))),
            $eta as f32,
        ));
    };
    ($texture:expr,$eta:expr) => {
        Material::Refract(Refract::new($texture, $eta as f32));
    };
}

#[macro_export]
macro_rules! emit {
    ($r:expr,$g:expr,$b:expr, $strength:expr) => {
        Material::Emit(Emit::new(
            Texture::SolidColour(SolidColour::new(colour!($r, $g, $b))),
            $strength as f32,
        ));
    };
    ($texture:expr,$strength:expr) => {
        Material::Emit(Emit::new($texture, $strength as f32));
    };
}

//-----
// PRIMITIVES
//-----
#[macro_export]
macro_rules! sphere {
    ($x:expr, $y:expr, $z:expr, $radius:expr, $material:expr) => {
        Primitive::Sphere(Sphere::new(
            position!($x, $y, $z),
            $radius as f32,
            $material,
        ))
    };
    ($position:expr, $radius:expr, $material:expr) => {
        Primitive::Sphere(Sphere::new($position, $radius as f32, $material))
    };
}

#[macro_export]
macro_rules! model {
    ($filepath:expr, $material:expr) => {
        load_model($filepath, $material)
    };
}

//-----
// OTHER STRUCTURES
//-----
#[macro_export]
macro_rules! sky {
    () => {
        Sky::new(None)
    };
    ($sky_texture:expr) => {
        Sky::new(Some($sky_texture))
    };
}

#[macro_export]
macro_rules! scene {
    ($origin:expr, $lookat:expr, $vup:expr, $fov:expr, $aspect_ratio:expr, $aperture:expr, $focus_dist:expr, $sky:expr, $split_type:expr, $primitives:expr) => {
        Scene::new(
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
