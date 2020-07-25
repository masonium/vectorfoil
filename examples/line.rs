use nalgebra_glm as glm;
use vectorfoil::Renderer;

use glm::vec3;

fn main() {
    let view = glm::look_at(
        &vec3(0.0_f64, 0.0, 5.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );
    let proj = glm::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 10.0);
    let clip = proj * view;

    let mut renderer = Renderer::new(&clip); //.cull_face(true);

    renderer.add_line(vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0));
    renderer.add_triangle(
        vec3(3.0, 0.0, 0.0),
        vec3(3.0, 1.0, 0.0),
        vec3(2.0, 0.0, 0.0),
    );
    renderer.add_triangle(
        vec3(-3.0, 0.0, 0.0),
        vec3(-3.0, 1.0, 0.0),
        vec3(-2.0, 0.0, 0.0),
    );

    let rp = renderer.render();

    println!("{:?}", rp);
}
