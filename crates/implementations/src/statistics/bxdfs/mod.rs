pub mod lambertian;
pub mod trowbridge_reitz;
pub mod trowbridge_reitz_vndf;

// All modules should adhere to the following:
// There should atleast provide the following functions
// sample(incoming, normal, ...)
// pdf(incoming, outgoing, normal, ...)
// If implementations of the above are provided in local space as well
// they must adhere to the same naming expect with _local and no normal parameter
// For the forementioned functions incoming will be pointing towards the surface
// outgoing will be pointing away from the surface and is the sampled direction
// Note that auxillary function do not have to adhere to the above
