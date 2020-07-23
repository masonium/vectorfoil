use nalgebra_glm as glm;
use vectorfoil::primitive::Tri;
use vectorfoil::intersect::split_triangle_by_segment;

use vectorfoil::EdgeType;

use glm::{vec2, vec4};

fn main() {
    let t = Tri {
        p: [
            vec4(0.0, 0.0, 0.0, 1.0),
            vec4(1.0, 0.0, 0.0, 1.0),
            vec4(0.0, 1.0, 0.0, 1.0),
        ],
        e: [EdgeType::Visible; 3],
    };

    let bc = vectorfoil::intersect::point_tri_comparison_test(vec2(0.5, -0.5), &t);
    println!("{:?}\n****", bc);

    println!(
        "{:?}\n****",
        split_triangle_by_segment(&t, vec2(0.5, -0.5), vec2(0.5, 0.5))
    );
    let s = split_triangle_by_segment(&t, vec2(0.5, -0.5), vec2(0.5, 0.0));
    println!("{:?}\n****", s);
    println!(
        "{:?}\n****",
        split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.5, 0.25))
    );
    println!(
        "{:?}\n****",
        split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.5, 0.5))
    );
    println!(
        "{:?}\n****",
        split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.0, 1.0))
    );
    println!(
        "{:?}\n****",
        split_triangle_by_segment(&t, vec2(0.0, 0.0), vec2(0.0, 1.0))
    );
}
