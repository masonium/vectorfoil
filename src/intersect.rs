use crate::common::*;
use crate::primitive::{EdgeType, Tri};
use std::ops::Not;

use glm::TMat3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayInt {
    Colinear,
    Parallel,
    Intersection(f32, f32),
}

impl RayInt {
    /// Return true iff the intersection represents a line-line
    /// intersection, rather than just a ray-ray intersection.
    fn is_line_line_isect(&self) -> bool {
        if let Self::Intersection(a, b) = self {
            inside_line_range(*a) && inside_line_range(*b)
        } else {
            false
        }
    }

    /// Return the t1 value if this is an intersection.
    #[allow(unused)]
    fn t1(&self) -> Option<f32> {
        if let Self::Intersection(a, _) = self {
            Some(*a)
        } else {
            None
        }
    }

    /// Return the t2 value if this is an intersection.
    fn t2(&self) -> Option<f32> {
        if let Self::Intersection(_, b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

impl From<&RayInt> for bool {
    fn from(rhs: &RayInt) -> bool {
        use RayInt::*;
        match rhs {
            Intersection(_, _) => true,
            _ => false,
        }
    }
}

impl Not for RayInt {
    type Output = bool;
    fn not(self) -> bool {
        let x: bool = (&self).into();
        !x
    }
}

#[derive(Clone, Debug)]
pub enum PointTriTest {
    /// The point is purely within the triangle.
    Inside(Vec3),

    /// The point is on edge i
    On(usize),

    /// The point is completely outside of the line.
    Outside,
}

/// Return true iff (p0, p1, p2) form a denegerate triangle.
pub fn is_degen_tri(p0: Vec2, p1: Vec2, p2: Vec2) -> bool {
    let p01 = p1 - p0;
    let p12 = p2 - p1;
    let signed_area = p01.x * p12.y - p01.y * p12.x;
    signed_area.abs() <= 1e-4 * (p01.norm() * p12.norm())
}

pub fn point_tri_comparison_test(p: Vec2, tri: &Tri) -> PointTriTest {
    if let Some(v) = barycentric_coords(p, tri) {
        if inside_line_range(v.x) && inside_line_range(v.y) && inside_line_range(v.z) {
            PointTriTest::Inside(v)
        } else if v.x < 0.0 || v.y < 0.0 || v.z < 0.0 {
            PointTriTest::Outside
        } else if on_line_range(v.x) {
            PointTriTest::On(1)
        } else if on_line_range(v.y) {
            PointTriTest::On(2)
        } else if on_line_range(v.z) {
            PointTriTest::On(0)
        } else {
            PointTriTest::Outside
        }
    } else {
        // If the matrix is not solvable, the triangle is colinear.
        PointTriTest::Outside
    }
}

pub fn barycentric_coords(p: Vec2, tri: &Tri) -> Option<Vec3> {
    let m: TMat3<f32> = TMat3::new(
        tri.p[0].x, tri.p[1].x, tri.p[2].x, tri.p[0].y, tri.p[1].y, tri.p[2].y, 1.0, 1.0, 1.0,
    );
    na::LU::new(m).solve(&vec3(p.x, p.y, 1.0))
}

/// Return the intersection point of two rays, each implicitly defined
/// by two points, assuming any finite t's are valid.
///
/// z-values are ignored.
pub fn implicit_ray_intersect_2d(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> RayInt {
    let da = a1 - a0;
    let db = b1 - b0;

    let m: TMat2<f32> = TMat2::new(da.x, -db.x, da.y, -db.y);

    match m.try_inverse() {
        Some(inv) => {
            let t = inv * (b0 - a0).xy();
            RayInt::Intersection(t.x, t.y)
        }
        None => {
            if is_degen_tri(a0, a1, b0) {
                RayInt::Colinear
            } else {
                RayInt::Parallel
            }
        }
    }
}

/// Return the intersection point along a and b if the lines
/// intersect. Otherwise, return colinear or no intersection as
/// appropriate.
///
/// z-values are ignored.
// pub fn line_intersect_2d(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> RayInt {
//     const EPS: f32 = 1e-4;
//     let isect = implicit_ray_intersect_2d(a0.xy(), a1.xy(), b0.xy(), b1.xy());
//     match isect {
// 	RayInt::Intersection(ta, tb) => {
//             if ta >= EPS && ta <= 1.0-EPS && tb >= EPS && tb <= 1.0-EPS {
//                 RayInt::Intersection(ta, tb)
//             } else {
//                 RayInt::NoIntersect
//             }
// 	}
// 	_ => isect
//     }
// }

/// Check whether the value is with the open interval (0, 1) using
/// some epsilon to decide the slack.
fn inside_line_range(t: f32) -> bool {
    const EPS: f32 = 1e-4;
    (t >= EPS) && t <= (1.0 - EPS)
}
fn on_line_range(t: f32) -> bool {
    const EPS: f32 = 1e-4;
    t.abs() < EPS || (1.0 - t).abs() < EPS
}

pub enum SplitResult<'a> {
    Original(&'a Tri),
    Split(Vec<Tri>),
}

impl<'a> From<&'a Tri> for SplitResult<'a> {
    fn from(tri: &'a Tri) -> SplitResult {
        SplitResult::Original(tri)
    }
}
impl<'a> From<Vec<Tri>> for SplitResult<'a> {
    fn from(v: Vec<Tri>) -> SplitResult<'a> {
        SplitResult::Split(v)
    }
}

/// Split a triangle based on a particular line segment.
///
/// # Remarks.
///
/// It is assumed within this function that p0-p1 is 'on top' of
/// `tri`. Triangles are not degenerate.
pub fn split_triangle_by_segment(tri: &Tri, p0: Vec2, p1: Vec2) -> SplitResult {
    let i0 = implicit_ray_intersect_2d(p0.xy(), p1.xy(), tri.p[0].xy(), tri.p[1].xy());
    let i1 = implicit_ray_intersect_2d(p0.xy(), p1.xy(), tri.p[1].xy(), tri.p[2].xy());
    let i2 = implicit_ray_intersect_2d(p0.xy(), p1.xy(), tri.p[2].xy(), tri.p[0].xy());

    let b0 = i0.is_line_line_isect();
    let b1 = i1.is_line_line_isect();
    let b2 = i2.is_line_line_isect();

    let isects = [i0, i1, i2];
    let is_line_line = [b0, b1, b2];

    for i in 0..3 {
        // Check for intersections with line boundaries.
        if is_line_line[i] {
            if let RayInt::Intersection(_, _) = isects[i] {
                return split_triangle_aux(tri, i, &isects).into();
            }
        }
    }
    // If any of the edges are colinear with the segment, we don't need to split
    if i0 == RayInt::Colinear || i1 == RayInt::Colinear || i2 == RayInt::Colinear {
        return tri.into();
    }

    let e0 = point_tri_comparison_test(p0, tri);
    let e1 = point_tri_comparison_test(p1, tri);

    match (e0, e1) {
        // If both points are outside, there is no splitting. (We've
        // already handled the case where the points form a line
        // segment that intersects two lines, since it necessarily
        // intersects at least one as above).
        (PointTriTest::Outside, PointTriTest::Outside) => tri.into(),
        (PointTriTest::Inside(_), PointTriTest::Inside(_)) => {
            // If both point are strict inside, pick the smallest
            // positive intersection as the edge. to split with
            let edge_isect = isects.iter().enumerate().filter_map(|(idx, isect)| {
		match isect {
		    RayInt::Intersection(t1, t2) if *t1 > 0.0 => {
			Some((idx, *t1, *t2))
		    }
		    _ => None
		}
	    }).min_by(|(_, u, _), (_, v, _)| u.partial_cmp(v).unwrap())
    	    .expect("An interior line segment should have at least one positive intersection with a triangle edge.");
            split_triangle_aux(tri, edge_isect.0, &isects).into()
        }
        (PointTriTest::On(e), PointTriTest::Inside(_))
        | (PointTriTest::On(e), PointTriTest::On(_)) => split_triangle_aux(tri, e, &isects).into(),
        (PointTriTest::Inside(_), PointTriTest::On(e)) => {
            split_triangle_aux(tri, e, &isects).into()
        }
        (PointTriTest::Outside, PointTriTest::On(_))
        | (PointTriTest::On(_), PointTriTest::Outside) => tri.into(),
        _ => {
            panic!("Inside/outside should be the only remaining case, and that should be covered.")
        }
    }
}

/// Assumes p{0, 1} are of the form (x/w, y/w, z/w, w) and computes
/// the interpolation along p0 -> p1 such that w fits.
fn perspective_lerp(t: f32, p0: Vec4, p1: Vec4) -> Vec4 {
    let p = art_util::easing::lerp(t, p0.xyz(), p1.xyz());
    let w = p0.w * p1.w / art_util::easing::lerp(t, p1.w, p0.w); // note the reversal; w(0) = p0.w
    vec4(p.x, p.y, p.z, w)
}

/// Split a triangle by a 'ray', where the intersection occurs along
/// edge i.
fn split_triangle_aux(tri: &Tri, e: usize, isects: &[RayInt]) -> Vec<Tri> {
    // find the interpolate point on the edge.
    let p = perspective_lerp(isects[e].t2().unwrap(), tri.p[e], tri.p[(e + 1) % 3]);
    let e1 = (e + 1) % 3;
    let e2 = (e + 2) % 3;

    //
    if let Some(t2) = isects[e1].t2() {
        if inside_line_range(t2) {
            let q = perspective_lerp(t2, tri.p[e1], tri.p[e2]);
            return vec![
                Tri {
                    p: [p, tri.p[e1], q],
                    e: [tri.e[e], tri.e[e1], EdgeType::Split],
                },
                Tri {
                    p: [p, q, tri.p[e2]],
                    e: [EdgeType::Split, tri.e[e1], EdgeType::Split],
                },
                Tri {
                    p: [p, tri.p[e2], tri.p[e]],
                    e: [EdgeType::Split, tri.e[e2], tri.e[e]],
                },
            ];
        }
    }
    if let Some(t2) = isects[e2].t2() {
        let q = perspective_lerp(t2, tri.p[e2], tri.p[e]);
        vec![
            Tri {
                p: [p, tri.p[e1], tri.p[e2]],
                e: [tri.e[e], tri.e[e1], EdgeType::Split],
            },
            Tri {
                p: [p, tri.p[e2], q],
                e: [EdgeType::Split, tri.e[e2], EdgeType::Split],
            },
            Tri {
                p: [p, q, tri.p[e]],
                e: [EdgeType::Split, tri.e[e2], tri.e[e]],
            },
        ];
    }

    // otherwise, we're hitting the opposite point
    vec![
        Tri {
            p: [p, tri.p[e1], tri.p[e2]],
            e: [tri.e[e], tri.e[e1], EdgeType::Split],
        },
        Tri {
            p: [p, tri.p[e2], tri.p[e]],
            e: [EdgeType::Split, tri.e[e2], tri.e[e]],
        },
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    pub fn test_cross() {
        const N: usize = 11;
        for t in 0..N {
            let dt: f32 = t as f32 / (N + 1) as f32;
            let dv = vec2(1.0, 1.0) * dt;

            let isect = line_intersect(
                vec2(-1.0, -1.0) + dv,
                vec2(1.0, 1.0) + dv,
                vec2(-1.0, 1.0),
                vec2(1.0, -1.0),
            );
            match isect {
                RayInt::Intersection(t1, t2) => {
                    assert_approx_eq!(t1, 0.5 * (1.0 - dt));
                    assert_approx_eq!(t2, 0.5);
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    pub fn test_parallel() {
        const N: usize = 11;
        for t in 0..N {
            let dt: f32 = t as f32 / (N + 1) as f32;
            let dv = vec2(1.0, 1.0) * dt;

            let isect = line_intersect(
                vec2(-1.0, -1.0) + dv,
                vec2(1.0, 1.0) + dv,
                vec2(-1.0, -1.0),
                vec2(1.0, 1.0),
            );

            assert_eq!(isect, RayInt::Parallel);
        }
    }
}