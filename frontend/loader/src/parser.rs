use nom::Finish;
use rt_core::Float;
use std::collections::HashMap;
use thiserror::Error;

/// What kind was parsed from the scene file. Variants match the initial keyword
/// used before the name of the object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
	Camera,
	Material,
	Primitive,
	Sky,
	Texture,
	Other,
}

/// An unowned value for a key in the scene. This could be a collection of three
/// floats or a string referring to a filename or such.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectValue<'a> {
	Num1(Float),
	Num2(Float, Float),
	Num3(Float, Float, Float),
	Text(&'a str),
}

/// An unowned object in the scene format. Contains the type of the object and
/// an optional name if provided. Each key-value pair is stored in a `HashMap`.
#[derive(Debug, Clone)]
pub struct Object<'a> {
	pub kind: ObjectKind,
	pub name: Option<&'a str>,
	pub values: HashMap<&'a str, ObjectValue<'a>>,
}

impl ObjectKind {
	pub fn is_camera(&self) -> bool {
		matches!(self, ObjectKind::Camera)
	}

	pub fn is_material(&self) -> bool {
		matches!(self, ObjectKind::Material)
	}

	pub fn is_primitive(&self) -> bool {
		matches!(self, ObjectKind::Primitive)
	}

	pub fn is_sky(&self) -> bool {
		matches!(self, ObjectKind::Sky)
	}

	pub fn is_texture(&self) -> bool {
		matches!(self, ObjectKind::Texture)
	}
}

impl<'a> Object<'a> {
	pub fn lookup(&self, key: &str) -> Option<ObjectValue<'a>> {
		self.values.get(key).cloned()
	}
}

impl<'a> Default for Object<'a> {
	fn default() -> Self {
		Self {
			kind: ObjectKind::Other,
			name: Option::default(),
			values: HashMap::default(),
		}
	}
}

/// Despite being listed as version 1, there is still more to be added
/// (probably). For example comments, and probably addition value types.
mod ver1 {
	use super::{Object, ObjectKind, ObjectValue};

	use nom::{
		branch::alt,
		bytes::complete::tag,
		character::complete::{
			alpha1, alphanumeric1, line_ending, multispace0, not_line_ending, space0, space1,
		},
		combinator::{map, opt, recognize},
		error::{ParseError, VerboseError},
		multi::{many0, many0_count},
		number::complete::double,
		sequence::{delimited, pair, preceded, terminated, tuple},
		IResult,
	};
	use rt_core::Float;
	use std::collections::HashMap;

	pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

	pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
		inner: F,
	) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
	where
		F: Fn(&'a str) -> IResult<&'a str, O, E>,
	{
		delimited(multispace0, inner, multispace0)
	}

	pub fn identifier(i: &str) -> Res<&str, &str> {
		recognize(pair(
			alt((alpha1, tag("_"))),
			many0_count(alt((alphanumeric1, tag("_")))),
		))(i)
	}

	pub fn value(i: &str) -> Res<&str, ObjectValue> {
		let f = |i| preceded(space0, double)(i);
		alt((
			map(tuple((f, f, f)), |(a, b, c)| {
				ObjectValue::Num3(a as Float, b as Float, c as Float)
			}),
			map(tuple((f, f)), |(a, b)| {
				ObjectValue::Num2(a as Float, b as Float)
			}),
			map(f, |a| ObjectValue::Num1(a as Float)),
			map(preceded(space0, not_line_ending), ObjectValue::Text),
		))(i)
	}

	pub fn keyvalue(i: &str) -> Res<&str, (&str, ObjectValue)> {
		tuple((delimited(space0, identifier, space1), value))(i)
	}

	pub fn values(i: &str) -> Res<&str, HashMap<&str, ObjectValue>> {
		delimited(
			ws(tag("(")),
			map(many0(terminated(keyvalue, line_ending)), |vec| {
				vec.into_iter().collect()
			}),
			ws(tag(")")),
		)(i)
	}

	pub fn objectkind(i: &str) -> Res<&str, ObjectKind> {
		alt((
			map(tag("camera"), |_| ObjectKind::Camera),
			map(tag("material"), |_| ObjectKind::Material),
			map(tag("primitive"), |_| ObjectKind::Primitive),
			map(tag("sky"), |_| ObjectKind::Sky),
			map(tag("texture"), |_| ObjectKind::Texture),
		))(i)
	}

	pub fn object(i: &str) -> Res<&str, Object> {
		map(
			tuple((objectkind, opt(preceded(space1, identifier)), values)),
			|(kind, name, values)| Object { kind, name, values },
		)(i)
	}

	pub fn parse(i: &str) -> Res<&str, Vec<Object>> {
		many0(ws(object))(i)
	}
}

fn by_version(i: &str) -> ver1::Res<&str, Vec<Object>> {
	use nom::{bytes::complete::tag, combinator::opt};

	match opt(ver1::ws(tag("#ver1")))(i) {
		Ok((o, Some(_))) => ver1::parse(o),
		Ok((o, _)) => ver1::parse(o),
		Err(e) => Err(e),
	}
}

/// Possible errors that can occur when parsing the scene file.
#[derive(Error, Debug)]
pub enum ParseError {
	/// No idea what went wrong, ask Milo probably
	#[error("no idea what wrong, parsing fucked up")]
	ParsingError,
}

pub fn from_str(src: &str) -> Result<Vec<Object>, ParseError> {
	match by_version(src).finish() {
		Ok((o, _)) if !o.is_empty() => Err(ParseError::ParsingError),
		Ok((_, o)) => Ok(o),
		Err(e) => {
			eprintln!("Error parsing scene file: {e:#?}");
			Err(ParseError::ParsingError)
		}
	}
}
