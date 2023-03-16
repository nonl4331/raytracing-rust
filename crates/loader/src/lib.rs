pub mod materials;
pub mod meshes;
pub mod misc;
pub mod obj;
pub mod parser;
pub mod primitives;
pub mod textures;

use implementations::rt_core::{Float, NoHit, Primitive, Scatter, Vec2, Vec3};
use implementations::{Camera, Texture};
use region::{Region, RegionRes, RegionUniqSlice};
use std::{collections::HashMap, fmt};
use thiserror::Error;

pub trait Load: Sized {
	/// Take a set of properties and load an object from, optionally also
	/// provide a resource name for this object. Such as a texture name
	/// or material ID.
	fn load(props: Properties) -> Result<(Option<String>, Self), LoadErr>;
}

#[derive(Default)]
pub struct Lookup {
	texture: HashMap<String, RegionRes<()>>,
	scatter: HashMap<String, RegionRes<()>>,
}

impl fmt::Debug for Lookup {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		f.debug_struct("Lookup")
			.field("texture", &format_args!("{:?}", self.texture.keys()))
			.field("scatter", &format_args!("{:?}", self.scatter.keys()))
			.finish()
	}
}

impl Lookup {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn texture_insert<T: Texture>(
		&mut self,
		name: &str,
		res: RegionRes<T>,
	) -> Option<RegionRes<T>> {
		let key = name.into();
		let res = unsafe { std::mem::transmute(res) };
		self.texture
			.insert(key, res)
			.map(|o| unsafe { std::mem::transmute(o) })
	}
	pub fn scatter_insert<S: Scatter>(
		&mut self,
		name: &str,
		res: RegionRes<S>,
	) -> Option<RegionRes<S>> {
		let key = name.into();
		let res = unsafe { std::mem::transmute(res) };
		self.scatter
			.insert(key, res)
			.map(|o| unsafe { std::mem::transmute(o) })
	}

	pub fn texture_lookup<T: Texture>(&self, name: &str) -> Option<RegionRes<T>> {
		self.texture
			.get(name)
			.map(|o| unsafe { std::mem::transmute(o.clone()) })
	}
	pub fn scatter_lookup<S: Scatter>(&self, name: &str) -> Option<RegionRes<S>> {
		self.scatter
			.get(name)
			.map(|o| unsafe { std::mem::transmute(o.clone()) })
	}
}

#[derive(Debug)]
pub struct Properties<'a> {
	lookup: &'a Lookup,
	name: Option<String>,
	props: HashMap<String, PropertiesValue>,
	autocast: bool,
}

#[derive(Debug)]
enum PropertiesValue {
	Vec3(Vec3),
	Vec2(Vec2),
	Float(Float),
	Text(String),
}

impl<'a> From<parser::ObjectValue<'a>> for PropertiesValue {
	fn from(u: parser::ObjectValue<'a>) -> Self {
		use parser::ObjectValue::*;
		match u {
			Num1(a) => Self::Float(a),
			Num2(a, b) => Self::Vec2(Vec2::new(a, b)),
			Num3(a, b, c) => Self::Vec3(Vec3::new(a, b, c)),
			Text(t) => Self::Text(t.into()),
		}
	}
}

impl<'a> Properties<'a> {
	pub fn new(lookup: &'a Lookup, object: &parser::Object) -> Self {
		Self {
			lookup,
			name: object.name.map(Into::into),
			props: object
				.values
				.iter()
				.map(|(&k, &v)| (k.into(), v.into()))
				.collect(),
			autocast: true,
		}
	}

	pub fn auto_cast(&mut self, doit: bool) {
		self.autocast = doit;
	}

	pub fn texture<T: Texture>(&self, name: &str) -> Option<RegionRes<T>> {
		self.lookup.texture_lookup(self.text(name)?)
	}
	pub fn scatter<S: Scatter>(&self, name: &str) -> Option<RegionRes<S>> {
		self.lookup.scatter_lookup(self.text(name)?)
	}
	pub fn lookup_material<S: Scatter>(&self, name: &str) -> Option<RegionRes<S>> {
		self.lookup.scatter_lookup(name)
	}
	pub fn vec3(&self, name: &str) -> Option<Vec3> {
		match self.props.get(name) {
			Some(PropertiesValue::Vec3(x)) => Some(*x),
			Some(PropertiesValue::Float(x)) if self.autocast => Some(*x * Vec3::one()),
			_ => None,
		}
	}
	pub fn vec2(&self, name: &str) -> Option<Vec2> {
		match self.props.get(name) {
			Some(PropertiesValue::Vec2(x)) => Some(*x),
			Some(PropertiesValue::Float(x)) if self.autocast => Some(*x * Vec2::one()),
			_ => None,
		}
	}
	pub fn float(&self, name: &str) -> Option<Float> {
		match self.props.get(name) {
			Some(PropertiesValue::Float(x)) => Some(*x),
			_ => None,
		}
	}
	pub fn text(&self, name: &str) -> Option<&str> {
		match self.props.get(name) {
			Some(PropertiesValue::Text(x)) => Some(x),
			_ => None,
		}
	}
	pub fn name(&mut self) -> Option<String> {
		self.name.take()
	}

	pub fn default_texture<T: Texture>(&self) -> RegionRes<T> {
		self.lookup
			.texture_lookup("__DEFAULT_TEX")
			.expect("default texture not loaded")
	}
	pub fn default_scatter<S: Scatter>(&self) -> RegionRes<S> {
		self.lookup
			.scatter_lookup("__DEFAULT_MAT")
			.expect("default material not loaded")
	}
}

#[derive(Error, Debug)]
pub enum LoadErr {
	#[error("failed to load a file for the given reason")]
	FileNotRead(std::path::PathBuf, std::io::Error),
	#[error("failed to parse the scene config for the given reason")]
	ParseError(parser::ParseError),
	#[error("missing required type for object")]
	MissingRequiredVariantType,
	#[error("missing required value for object")]
	MissingRequired(String),
	#[error("missing required camera object")]
	MissingCamera,
	#[error("unknown error")]
	Any(Box<dyn std::error::Error>),
}

pub fn load_file_full<'a, T, M, P, C, S>(
	region: &'a mut Region,
	file: &str,
) -> Result<(RegionUniqSlice<'a, P>, C, S), LoadErr>
where
	T: Texture + Load,
	M: Scatter + Load,
	P: Primitive + Load + Clone,
	C: Camera + Load,
	S: NoHit + Load,
	Vec<P>: Load,
{
	let scene_file = match std::fs::read_to_string(file) {
		Ok(s) => s,
		Err(e) => return Err(LoadErr::FileNotRead(file.into(), e)),
	};
	log::debug!("Parsing scene file {}", file);
	let scene_conf = match parser::from_str(&scene_file) {
		Ok(c) => c,
		Err(e) => return Err(LoadErr::ParseError(e)),
	};

	let mut lookup = Lookup::new();

	log::info!("Loading textures...");
	let textures = load_textures::<T>(&scene_conf, &lookup)?;

	region_insert_with_lookup(region, textures, |n, t| lookup.texture_insert(n, t));

	log::info!("Loading materials...");
	let materials = load_materials::<M>(&scene_conf, &lookup)?;

	region_insert_with_lookup(region, materials, |n, s| lookup.scatter_insert(n, s));

	log::info!("Loading primitives...");
	let primitives = {
		let mut primitives = load_primitives::<P>(&scene_conf, &lookup)?;
		log::info!("Loading meshes...");
		primitives.extend(load_meshes::<P>(&scene_conf, &lookup)?);
		region.alloc_slice(&primitives)
	};

	log::info!("Loading other objects...");
	let camera = load_scene_camera(&scene_conf, &lookup)?;
	let sky = load_scene_sky(&scene_conf, &lookup)?;

	Ok((primitives, camera, sky))
}

pub fn load_str_full<'a, T, M, P, C, S>(
	region: &'a mut Region,
	data: &str,
) -> Result<(RegionUniqSlice<'a, P>, C, S), LoadErr>
where
	T: Texture + Load,
	M: Scatter + Load,
	P: Primitive + Load + Clone,
	C: Camera + Load,
	S: NoHit + Load,
	Vec<P>: Load,
{
	let scene_conf = match parser::from_str(data) {
		Ok(c) => c,
		Err(e) => return Err(LoadErr::ParseError(e)),
	};

	let mut lookup = Lookup::new();

	log::info!("Loading textures...");
	let textures = load_textures::<T>(&scene_conf, &lookup)?;

	region_insert_with_lookup(region, textures, |n, t| lookup.texture_insert(n, t));

	log::info!("Loading materials...");
	let materials = load_materials::<M>(&scene_conf, &lookup)?;

	region_insert_with_lookup(region, materials, |n, s| lookup.scatter_insert(n, s));

	log::info!("Loading primitives...");
	let primitives = {
		let mut primitives = load_primitives::<P>(&scene_conf, &lookup)?;
		log::info!("Loading meshes...");
		primitives.extend(load_meshes::<P>(&scene_conf, &lookup)?);
		region.alloc_slice(&primitives)
	};

	log::info!("Loading other objects...");
	let camera = load_scene_camera(&scene_conf, &lookup)?;
	let sky = load_scene_sky(&scene_conf, &lookup)?;

	Ok((primitives, camera, sky))
}

pub fn load_scene_camera<C>(objects: &[parser::Object], lookup: &Lookup) -> Result<C, LoadErr>
where
	C: Camera + Load,
{
	// Find a camera object
	let props = Properties::new(
		lookup,
		objects
			.iter()
			.find(|o| o.kind.is_camera())
			.ok_or(LoadErr::MissingCamera)?,
	);
	Ok(C::load(props)?.1)
}

pub fn load_scene_sky<S>(objects: &[parser::Object], lookup: &Lookup) -> Result<S, LoadErr>
where
	S: NoHit + Load,
{
	// Find a sky object, if none warn and use a default
	let obj = objects.iter().find(|o| o.kind.is_sky());
	let props = match obj {
		Some(o) => Properties::new(lookup, o),
		None => {
			log::warn!("no sky object was provided in scene file, using default");
			Properties::new(lookup, &Default::default())
		}
	};
	Ok(S::load(props)?.1)
}

fn region_insert_with_lookup<T: Sync>(
	region: &mut Region,
	items: Vec<(Option<String>, T)>,
	mut insert_fn: impl FnMut(&str, RegionRes<T>) -> Option<RegionRes<T>>,
) {
	//let block_size = std::alloc::Layout::array::<T>(items.len()).unwrap().size();
	//let block = region.allocate_block(block_size);
	for (name, item) in items.into_iter() {
		let uniq = region.alloc(item);
		if let Some(name) = name {
			if insert_fn(&name, uniq.shared()).is_some() {
				log::warn!("Overwrote previous object of name: '{name}'");
			}
		}
	}
}

fn load_textures<T: Texture + Load>(
	objects: &[parser::Object],
	lookup: &Lookup,
) -> Result<Vec<(Option<String>, T)>, LoadErr> {
	let mut textures = Vec::new();
	for obj in objects.iter().filter(|o| o.kind.is_texture()) {
		let props = Properties::new(lookup, obj);
		textures.push(<T as Load>::load(props)?);
	}
	// Load default texture, assumes that T contains SolidColor
	{
		use parser::{Object, ObjectKind, ObjectValue};
		let def_obj = Object {
			kind: ObjectKind::Texture,
			name: Some("__DEFAULT_TEX"),
			values: [
				("type", ObjectValue::Text("solid")),
				("colour", ObjectValue::Num1(1.0)),
			]
			.into(),
		};
		let props = Properties::new(lookup, &def_obj);
		textures.push(<T as Load>::load(props)?);
	}
	Ok(textures)
}

fn load_materials<S: Scatter + Load>(
	objects: &[parser::Object],
	lookup: &Lookup,
) -> Result<Vec<(Option<String>, S)>, LoadErr> {
	let mut materials = Vec::new();
	for obj in objects.iter().filter(|o| o.kind.is_material()) {
		let props = Properties::new(lookup, obj);
		materials.push(<S as Load>::load(props)?);
	}
	// Load default material, assumes that S contains Lambertian
	{
		use parser::{Object, ObjectKind, ObjectValue};
		let def_obj = Object {
			kind: ObjectKind::Material,
			name: Some("__DEFAULT_MAT"),
			values: [
				("type", ObjectValue::Text("lambertian")),
				("texture", ObjectValue::Text("__DEFAULT_TEX")),
				("albedo", ObjectValue::Num1(0.25)),
			]
			.into(),
		};
		let props = Properties::new(lookup, &def_obj);
		materials.push(<S as Load>::load(props)?);
	}
	Ok(materials)
}

fn load_primitives<P: Primitive + Load>(
	objects: &[parser::Object],
	lookup: &Lookup,
) -> Result<Vec<P>, LoadErr> {
	let mut primitives = Vec::new();
	for obj in objects.iter().filter(|o| o.kind.is_primitive()) {
		let props = Properties::new(lookup, obj);
		primitives.push(<P as Load>::load(props)?.1);
	}
	Ok(primitives)
}

fn load_meshes<P: Primitive + Load>(
	objects: &[parser::Object],
	lookup: &Lookup,
) -> Result<Vec<P>, LoadErr>
where
	Vec<P>: Load,
{
	let mut primitives = Vec::new();
	for obj in objects.iter().filter(|o| o.kind.is_mesh()) {
		let props = Properties::new(lookup, obj);
		primitives.extend(<Vec<P> as Load>::load(props)?.1);
	}
	Ok(primitives)
}

#[cfg(test)]
mod tests {
	use super::*;

	use implementations::*;

	const DATA: &str = "camera (
	origin   -5 3 -3
	lookat   0 0.5 0
	vup      0 1 0
	fov      34.0
	aperture 0.0
	focus_dis 10.0
)

texture sky (
	type solid
	colour 0.0
)

sky (
	texture sky
)

texture grey (
	type solid
	colour 0.5
)

texture white (
	type solid
	colour 1.0
)

material ground (
	type lambertian
	texture grey
	albedo 0.5
)

material light (
	type emissive
	texture white
	strength 1.5
)

primitive (
	type sphere
	material ground
	centre 0 -1000 0
	radius 1000
)

primitive (
	type sphere
	material light
	centre 0 0.5 0
	radius 0.5
)

primitive (
	type sphere
	material ground
	centre -0.45 0.15 -0.45
	radius 0.05
)";

	#[test]
	fn scene() {
		let mut region = Region::new();
		type Tex = AllTextures;
		type Mat<'a> = AllMaterials<'a, Tex>;
		type Prim<'a> = AllPrimitives<'a, Mat<'a>>;
		type SkyType<'a> = Sky<'a, Tex>;
		let stuff =
			load_str_full::<Tex, Mat, Prim, SimpleCamera, SkyType>(&mut region, DATA).unwrap();

		let (p, _, _) = stuff;
		let _: Bvh<Prim, Mat> = Bvh::new(p, split::SplitType::Sah);
	}
}
