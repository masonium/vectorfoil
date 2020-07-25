use glm::{look_at, perspective, vec3};
use nalgebra_glm as glm;
use vectorfoil::Renderer;

fn main() -> std::io::Result<()> {
    let dpi = 72.0;
    let width = 10.0;
    let height = 10.0;

    let view = look_at(
        &vec3(1.5, 2.0, 3.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );
    let proj = perspective(width / height, std::f64::consts::FRAC_PI_2, 1.0, 9.0);
    let mut renderer = Renderer::new(&(proj * view));

    // front
    renderer.add_polygon(&[
        vec3(1.0, 1.0, 1.0),
        vec3(-1.0, 1.0, 1.0),
        vec3(-1.0, -1.0, 1.0),
        vec3(1.0, -1.0, 1.0),
    ]);
    for i in 1..=5 {
        let z = 1.0 + i as f64 * 0.2;
        renderer.add_polygon(&[vec3(1.0, 1.0, z), vec3(-1.0, 1.0, z), vec3(-1.0, -1.0, z)]);
    }
    // back
    renderer.add_polygon(&[
        vec3(1.0, -1.0, -1.0),
        vec3(-1.0, -1.0, -1.0),
        vec3(-1.0, 1.0, -1.0),
        vec3(1.0, 1.0, -1.0),
    ]);

    // right
    renderer.add_polygon(&[
        vec3(1.0, 1.0, -1.0),
        vec3(1.0, 1.0, 1.0),
        vec3(1.0, -1.0, 1.0),
        vec3(1.0, -1.0, -1.0),
    ]);

    // // left
    renderer.add_polygon(&[
        vec3(-1.0, -1.0, -1.0),
        vec3(-1.0, -1.0, 1.0),
        vec3(-1.0, 1.0, 1.0),
        vec3(-1.0, 1.0, -1.0),
    ]);

    // // top
    renderer.add_polygon(&[
        vec3(1.0, 1.0, -1.0),
        vec3(-1.0, 1.0, -1.0),
        vec3(-1.0, 1.0, 1.0),
        vec3(1.0, 1.0, 1.0),
    ]);

    // bottom
    renderer.add_polygon(&[
        vec3(1.0, -1.0, -1.0),
        vec3(-1.0, -1.0, -1.0),
        vec3(-1.0, -1.0, 1.0),
        vec3(1.0, -1.0, 1.0),
    ]);

    let rp = renderer.render();
    let d = vectorfoil::render_paths::standalone_svg(&rp, width, height, dpi);

    svg::save("cube.svg", &d)
}
