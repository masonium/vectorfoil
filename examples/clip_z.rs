use nalgebra_glm as glm;
use vectorfoil::{Renderer, Primitive, EdgeType};
use vectorfoil::intersect::{barycentric_coords, split_triangle_by_segment, implicit_ray_intersect_2d, Tri};

use glm::{vec2, vec3, vec4, Vec3};

fn proj(v: Vec3) -> Vec3 {
    let view = glm::look_at(&vec3(0.0, 0.0, 5.0),
			    &vec3(0.0, 0.0, 0.0),
			    &vec3(0.0, 1.0, 0.0));
    let proj = glm::perspective(1.0, std::f32::consts::FRAC_PI_2, 0.1, 10.0);
    let r = proj * view * vec4(v.x, v.y, v.z, 1.0);
    r.xyz() / r.w
}

fn main() {
    let t = Tri {
	p: [vec4(0.0, 0.0, 0.0, 1.0),
	    vec4(1.0, 0.0, 0.0, 1.0),
	    vec4(0.0, 1.0, 0.0, 1.0)],
	e: [EdgeType::Visible; 3]
    };

    let bc = vectorfoil::intersect::point_tri_comparison_test(vec2(0.5, -0.5), &t);
    println!("{:?}\n****", bc);

    println!("{:?}\n****", split_triangle_by_segment(&t, vec2(0.5, -0.5), vec2(0.5, 0.5)));
    let s = split_triangle_by_segment(&t, vec2(0.5, -0.5), vec2(0.5, 0.0));
    println!("{:?}\n****", s);
    println!("{:?}\n****", split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.5, 0.25)));
    println!("{:?}\n****", split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.5, 0.5)));
    println!("{:?}\n****", split_triangle_by_segment(&t, vec2(0.5, 0.0), vec2(0.0, 1.0)));
    println!("{:?}\n****", split_triangle_by_segment(&t, vec2(0.0, 0.0), vec2(0.0, 1.0)));

}
