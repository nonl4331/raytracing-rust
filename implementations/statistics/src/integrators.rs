use crate::{Float, Vec3};
const MAX_DEPTH: usize = 6;
const EPSILON: Float = 0.000001;

pub fn integrate_solid_angle<F>(
	pdf: &F,
	theta_start: Float,
	theta_end: Float,
	phi_start: Float,
	phi_end: Float,
) -> Float
where
	F: Fn(Vec3) -> Float,
{
	let pdf = |phi: Float, a, b| {
		adaptive_simpsons(
			|theta| {
				pdf(Vec3::new(
					phi.cos() * theta.sin(),
					phi.sin() * theta.sin(),
					theta.cos(),
				)) * theta.sin()
			},
			a,
			b,
		)
	};

	adaptive_simpsons(|phi| pdf(phi, theta_start, theta_end), phi_start, phi_end)
}

pub fn adaptive_simpsons<F>(function: F, a: Float, b: Float) -> Float
where
	F: Fn(Float) -> Float,
{
	fn aux<F>(
		function: &F,
		a: Float,
		b: Float,
		c: Float,
		fa: Float,
		fb: Float,
		fc: Float,
		i: Float,
		epsilon: Float,
		depth: usize,
	) -> Float
	where
		F: Fn(Float) -> Float,
	{
		let d = 0.5 * (a + b);
		let e = 0.5 * (b + c);
		let fd = function(d);
		let fe = function(e);

		let h = c - a;
		let i0 = (1.0 / 12.0) * h * (fa + 4.0 * fd + fb);
		let i1 = (1.0 / 12.0) * h * (fb + 4.0 * fe + fc);
		let ip = i0 + i1;

		if depth >= MAX_DEPTH || (ip - i).abs() < 15.0 * epsilon {
			return ip + (1.0 / 15.0) * (ip - i);
		}

		aux(function, a, d, b, fa, fd, fb, i0, 0.5 * epsilon, depth + 1)
			+ aux(function, b, e, c, fb, fe, fc, i1, 0.5 * epsilon, depth + 1)
	}
	let c = b;
	let b = 0.5 * (a + b);

	let fa = function(a);
	let fb = function(b);
	let fc = function(c);
	let i = (c - a) * (1.0 / 6.0) * (fa + 4.0 * fb + fc);
	aux(&function, a, b, c, fa, fb, fc, i, EPSILON, 0)
}
