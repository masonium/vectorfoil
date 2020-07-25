pub use nalgebra as na;
pub use nalgebra_glm as glm;

pub use glm::{vec2, vec3, vec4, TMat2, Vec2, Vec3, Vec4};
pub use glm::{DVec2, DVec3, DVec4};

pub use crate::primitive::{EdgeType, Primitive};

pub(crate) const EPS: f64 = 1e-5;
pub(crate) const LINE_LENGTH_EPS: f64 = 1e-5;
