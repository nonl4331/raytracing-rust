use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Scatter)]
pub fn derive_scatter(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);
	let enum_name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let fields = match input.data {
		Data::Enum(e) => e.variants,
		_ => {
			panic!("#[derive(Scatter)] only works on enums!")
		}
	};

	let func_names = [
		(
			quote!(scatter_ray(&self, __one: &mut Ray, __two: &Hit) -> bool),
			quote!(scatter_ray(__one, __two)),
		),
		(quote!(requires_uv(&self) -> bool), quote!(requires_uv())),
		(quote!(is_light(&self) -> bool), quote!(is_light())),
		(quote!(ls_chance(&self) -> Float), quote!(ls_chance())),
		(quote!(is_delta(&self) -> bool), quote!(is_delta())),
		(
			quote!(scattering_pdf(&self, __one: &Hit, __two: Vec3, __three: Vec3) -> Float),
			quote!(scattering_pdf(__one, __two, __three)),
		),
		(
			quote!(eval(&self, __one: &Hit, __two: Vec3, __three: Vec3) -> Vec3),
			quote!(eval(__one, __two, __three)),
		),
		(
			quote!(get_emission(&self, __one: &Hit, __two: Vec3) -> Vec3),
			quote!(get_emission(__one, __two)),
		),
	]
	.into_iter();

	let variant_names = fields
		.iter()
		.map(move |field| &field.ident)
		.collect::<Vec<_>>();

	let functions = func_names.map(|(f_name, f_used)| {
		quote! {
			fn #f_name {
				match self {
					#( #enum_name::#variant_names (a) => a.#f_used, )*
				}
			}
		}
	});

	quote! {
		impl #impl_generics Scatter for #enum_name #ty_generics #where_clause {#( #functions )*}
	}
	.into()
}

#[proc_macro_derive(Texture)]
pub fn derive_texture(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);
	let enum_name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let fields = match input.data {
		Data::Enum(e) => e.variants,
		_ => {
			panic!("#[derive(Texture)] only works on enums!")
		}
	};

	let func_names = [
		(
			quote!(colour_value(&self, __one: Vec3, __two: Vec3) -> Vec3),
			quote!(colour_value(__one, __two)),
		),
		(quote!(requires_uv(&self) -> bool), quote!(requires_uv())),
	]
	.into_iter();

	let variant_names = fields
		.iter()
		.map(move |field| &field.ident)
		.collect::<Vec<_>>();

	let functions = func_names.map(|(f_name, f_used)| {
		quote! {
			fn #f_name {
				match self {
					#( #enum_name::#variant_names (a) => a.#f_used, )*
				}
			}
		}
	});

	quote! {
		impl #impl_generics Texture for #enum_name #ty_generics #where_clause {#( #functions )*}
	}
	.into()
}

#[proc_macro_derive(Primitive)]
pub fn derive_primitive(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);
	let enum_name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let fields = match input.data {
		Data::Enum(e) => e.variants,
		_ => {
			panic!("#[derive(Primitive)] only works on enums!")
		}
	};

	let func_names_primitive = [
		(
			quote!(get_int(&self, __one: &Ray) -> Option<SurfaceIntersection #ty_generics>),
			quote!(get_int(__one)),
		),
		(
			quote!(does_int(&self, __one: &Ray) -> bool),
			quote!(does_int(__one)),
		),
		(
			quote!(get_uv(&self, __one: Vec3) -> Option<Vec2>),
			quote!(get_uv(__one)),
		),
		(quote!(get_sample(&self) -> Vec3), quote!(get_sample())),
		(
			quote!(sample_visible_from_point(&self, __one: Vec3) -> Vec3),
			quote!(sample_visible_from_point(__one)),
		),
		(quote!(area(&self) -> Float), quote!(area())),
		(
			quote!(scattering_pdf(&self, __one: &Hit, __two: Vec3, __three: Vec3) -> Float),
			quote!(scattering_pdf(__one, __two, __three)),
		),
		(
			quote!(material_is_light(&self) -> bool),
			quote!(material_is_light()),
		),
	]
	.into_iter();

	let func_name_aabound = [(quote!(get_aabb(&self) -> AABB), quote!(get_aabb()))];

	let variant_names = fields
		.iter()
		.map(move |field| &field.ident)
		.collect::<Vec<_>>();

	let functions_primitive = func_names_primitive.map(|(f_name, f_used)| {
		quote! {
			fn #f_name {
				match self {
					#( #enum_name::#variant_names (a) => a.#f_used, )*
				}
			}
		}
	});
	let functions_aabound = func_name_aabound.map(|(f_name, f_used)| {
		quote! {
			fn #f_name {
				match self {
					#( #enum_name::#variant_names (a) => a.#f_used, )*
				}
			}
		}
	});

	quote! {
		impl #impl_generics Primitive #ty_generics for #enum_name #ty_generics #where_clause {#( #functions_primitive )*}
		impl #impl_generics AABound for #enum_name #ty_generics #where_clause { #( #functions_aabound )*}
	}
	.into()
}
