mod common;
pub mod intersect;
pub mod primitive;
pub mod renderer;
pub mod render_paths;

pub use primitive::{EdgeType, Primitive};
pub use render_paths::{RenderPaths, RenderLine};
//use primitive::ZsortPrim;
pub use renderer::Renderer;
