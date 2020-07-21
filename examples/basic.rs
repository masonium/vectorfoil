use vectorfoil::{Renderer, Primitive};
use nalgebra_glm as glm;
use glm::{vec3, vec4, look_at, perspective};

fn main() {
    let view = look_at(&vec3(0.0, 0.0, 2.0),
		       &vec3(0.0, 0.0, 0.0),
		       &vec3(0.0, 1.0, 0.0));
    let proj = perspective(1.0, std::f32::consts::FRAC_PI_2, 1.0, 9.0);
    let mut renderer = Renderer::new(&(proj * view));

    renderer.add_tri(&vec3(-1.0, -1.0, 1.0),
		     &vec3(0.5, 0.0, 1.0),
		     &vec3(-1.0, 1.0, 1.0));
    renderer.add_tri(&vec3(-0.5, 0.0, 1.0),
		     &vec3(1.0, -1.0, 1.0),
		     &vec3(1.0, 1.0, 1.0));

    println!("{:?}", renderer.render());
}
