use glm::{vec2, vec4};
use nalgebra_glm as glm;
use vectorfoil::{split_triangle_by_segment, EdgeType, Tri};

fn main() {
    let v = [
	vec4(0.1888026940, -0.0181302809, 0.0, 1.0),
	vec4(-0.0983901363, 0.3401342839, 0.0, 1.0),
	vec4(-0.0802953986, -0.0308423169, 0.0, 1.0),
    ];
    let p0 = vec2(0.1580475040, 0.0202359093);
    let p1 = vec2(0.1888026940, -0.0181302809);

    let tri = Tri {
	p: v,
	e: [EdgeType::Visible; 3],
    };
    split_triangle_by_segment(&tri, p0, p1);

    // let v1 = [vec4(0.1888026940, -0.0181302809, 0.0, 1.0),
    //        vec4(0.1958607387, 0.0438854463, 0.0, 1.0),
    //        vec4(0.1425652996, 0.0395495016, 0.0, 1.0)];
    // let v2 = [vec4(0.1995914194, 0.0766651878, 0.0, 1.0),
    //        vec4(-0.3326523656, -0.2697478830, 0.0, 1.0),
    //        vec4(0.1369745035, -0.4735202776, 0.0, 1.0)];

    // let tri1 = Tri {p: v1, e: [EdgeType::Visible; 3]};
    // let tri2 = Tri {p: v2, e: [EdgeType::Visible; 3]};

    // triangle_in_triangle_2d(&tri1, &tri2);
}
