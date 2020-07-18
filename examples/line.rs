use nalgebra as na;
use nalgebra_glm as glm;
use vectorfoil::{EdgeType, Primitive, Renderer};

use glm::{vec2, vec3};

type V3 = na::Vector3<f32>;

fn main() {
    let view = glm::look_at(
        &vec3(0.0, 0.0, 5.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );
    let proj = glm::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 10.0);
    let clip = proj * view;

    let mut renderer = vectorfoil::Renderer::new(&clip).cull(true);

    renderer.add_prim(Primitive::Line {
        points: [vec3(-1.0, -1.0, 0.0), vec3(1.0, 1.0, 0.0)],
    });
    renderer.add_prim(Primitive::Polygon {
        points: vec![
            vec3(3.0, 0.0, 0.0),
            vec3(3.0, 1.0, 0.0),
            vec3(2.0, 0.0, 0.0),
        ],
        edges: vec![EdgeType::Visible; 3],
    });
    renderer.add_prim(Primitive::Polygon {
        points: vec![
            vec3(-3.0, 0.0, 0.0),
            vec3(-3.0, 1.0, 0.0),
            vec3(-2.0, 0.0, 0.0),
        ],
        edges: vec![EdgeType::Visible; 3],
    });

    let rp = renderer.render();

    println!("{:?}", rp);
}
