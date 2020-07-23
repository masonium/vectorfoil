mod common;
pub mod intersect;
pub mod primitive;
pub mod render_paths;
pub mod renderer;

pub use intersect::{split_triangle_by_segment, triangle_in_triangle_2d};
pub use primitive::{EdgeType, Primitive, Tri};
pub use render_paths::{standalone_svg, RenderLine, RenderPaths};
//use primitive::ZsortPrim;
pub use renderer::Renderer;
