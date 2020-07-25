//! regression tests, mostly for split_triangle_by_segment
use glm::{vec2, vec4};
use nalgebra_glm as glm;
use vectorfoil::{split_triangle_by_segment, EdgeType, Tri};

#[test]
fn split_tri() {
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

    let v = [
        vec4(0.1970887057, 0.0546750164, 0.0, 1.0),
        vec4(0.1888026940, -0.0181302809, 0.0, 1.0),
        vec4(0.1528945590, 0.0266640559, 0.0, 1.0),
    ];
    let p0 = vec2(0.1714282130, 0.0384109837);
    let p1 = vec2(0.1970887057, 0.0546750164);

    let tri = Tri {
        p: v,
        e: [EdgeType::Visible; 3],
    };
    split_triangle_by_segment(&tri, p0, p1);

    let v = [
        vec4(0.1528945590039597, 0.026664055882639027, 0.0, 1.0),
        vec4(0.15289455900395973, 0.02666405588263901, 0.0, 1.0),
        vec4(0.1970887056666544, 0.05467501637773782, 0.0, 1.0),
    ];
    let p0 = vec2(0.15825014652177105, 0.030058513332726834);
    let p1 = vec2(0.19404721329525543, 0.027950849718747416);
    let tri = Tri {
        p: v,
        e: [EdgeType::Visible; 3],
    };
    split_triangle_by_segment(&tri, p0, p1);
}
