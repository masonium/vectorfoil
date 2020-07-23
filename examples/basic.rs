use glm::{look_at, perspective, vec3};
use nalgebra_glm as glm;
use svg::Document;
use vectorfoil::Renderer;

fn main() {
    let dpi = 72.0;
    let width = 10.0;
    let height = 7.0;
    let mut d = svg::Document::new()
        .set("width", format!("{}", width * dpi))
        .set("height", format!("{}", height * dpi))
        .add(svg::node::element::Style::new(
            ".visible { stroke-width: 0.01; fill: none; stroke: #444444; }
.hidden { stroke-width: 0.001; fill: none; stroke: #2222cc; stroke-dasharray: 0.001 0.001; }
.invisible { stroke-width: 0.001; fill: none; stroke: #888888; stroke-dasharray: 0.002 002; }
.split { stroke-width: 0.001; fill: none; stroke: #888888; stroke-dasharray: 0.005 0.005; }
.culled { stroke-width: 0.001; fill: none; stroke: #cc2222; stroke-dasharray: 0.005 0.005; }",
        ));

    let view = look_at(
        &vec3(0.0, 0.0, 4.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );
    let proj = perspective(width / height, std::f64::consts::FRAC_PI_2, 1.0, 9.0);
    let mut renderer = Renderer::new(&(proj * view));

    renderer.add_triangle(
        &vec3(-1.0, -1.0, 1.0),
        &vec3(0.5, 0.0, 1.0),
        &vec3(-1.0, 1.0, 1.0),
    );
    renderer.add_triangle(
        &vec3(-0.5, 0.0, -1.0),
        &vec3(1.0, -1.0, -1.0),
        &vec3(1.0, 1.0, -1.0),
    );

    let mut g = svg::node::element::Group::new().set(
        "transform",
        format!(
            "translate({} {}) scale({} -{})",
            width * dpi / 2.0,
            height * dpi / 2.0,
            width * dpi / 2.0,
            height * dpi / 2.0
        ),
    );

    let r = renderer.render();

    println!("{:?}", r);

    g = g.add(r.as_svg());
    d = d.add(g);

    svg::save("x.svg", &d);
}
