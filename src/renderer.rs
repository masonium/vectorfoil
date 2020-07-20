use na::Matrix4;

use crate::common::*;
use crate::primitive::*;
//use nalgebra_glm as glm;

/// Output line from the `Renderer`.
#[derive(Debug, Clone, Copy)]
pub struct RenderLine {
    points: [Vec2; 2],
    edge: EdgeType,
}

impl RenderLine {
    pub fn new(p0: &Vec2, p1: &Vec2, e: EdgeType) -> RenderLine {
        RenderLine {
            points: [*p0, *p1],
            edge: e,
        }
    }
}

/// Rendering output from the `Renderer`.
#[derive(Debug, Clone, Default)]
pub struct RenderPaths {
    pub points: Vec<Vec2>,
    pub hidden_points: Vec<Vec2>,

    pub lines: Vec<RenderLine>,
}

impl RenderPaths {
    /// Return true iff there are no pieces to render, visible or
    /// hidden.
    pub fn is_empty(&self) -> bool {
	self.points.is_empty() && self.hidden_points.is_empty() && self.lines.is_empty()
    }
}

pub struct Renderer {
    clip: Matrix4<f32>,
    input_primitives: Vec<Primitive>,
    depth_range: [f32; 2]
}

impl Renderer {
    pub fn new(c: &Matrix4<f32>) -> Renderer {
        Renderer {
            clip: *c,
            input_primitives: vec![],
	    depth_range: [-1.0, 1.0]
        }
    }

    /// add a primitive to the render list
    pub fn add_prim(&mut self, p: Primitive) {
        self.input_primitives.push(p);
    }

    /// Add a point to the list, given the primitive.
    pub fn add_point(&mut self, p: Vec3) {
	self.add_prim(Primitive::Point { point: vec4(p.x, p.y, p.z, 1.0) });
    }

    /// Project the primitive as a whole.
    fn proj_prim(&self, prim: &Primitive) -> Primitive {
        match prim {
	    Primitive::Point { point } => {
		Primitive::Point { point: self.proj(&point) }
	    },
	    Primitive::Line { points } => {
		Primitive::Line {
		    points: [self.proj(&points[0]), self.proj(&points[1])]
		}
	    },
	    Primitive::Triangle { points, edges } => {
		Primitive::Triangle {
		    points: [self.proj(&points[0]), self.proj(&points[1]), self.proj(&points[2])],
		    edges: edges.clone()
		}
	    }
	}
    }
    
    /// Project a point into NDC.
    fn proj(&self, p: &Vec4) -> Vec4 {
        let r = self.clip * na::Vector4::new(p.x, p.y, p.z, 1.0);
        Vec4::new(r.x / r.w, r.y / r.w, r.z / r.w, r.w)
    }

    /// Return true if a (projected) primitve can be trivially clipped
    /// from the viewing frustum.
    fn is_prim_clipped(&self, prim: &Primitive) -> bool {
	match prim {
	    Primitive::Point { point } => {
		self.points_clipped(&[*point])
	    }
	    Primitive::Line { points } => {
		self.points_clipped(points)
	    },
	    Primitive::Triangle { points, .. } => {
		self.points_clipped(points)
	    }
	}
    }

    /// Return true iff the set of input `points` can be
    /// conservatively clipped because they are all on the wrong side
    /// of a single clipping plane.
    fn points_clipped(&self, points: &[Vec4]) -> bool {
	// Check against each of the 6 clipping planes. If the set of
	// is on the wrong side of any plane, the primitive
	points.iter().all(|v| v.x < -1.0) ||
	    points.iter().all(|v| v.x > 1.0) ||
	    points.iter().all(|v| v.y < -1.0) ||
	    points.iter().all(|v| v.y > 1.0) ||
	    points.iter().all(|v| v.z < self.depth_range[0]) ||
	    points.iter().all(|v| v.z > self.depth_range[1])
    }

    pub fn render(&self) -> RenderPaths {
        let mut rp = RenderPaths::default();

	// project the primitives
	let projected: Vec<_> = self.input_primitives
	    .iter()
	    .map(|p| self.proj_prim(p))
	    .collect();

	let clipped: Vec<_> = projected
	    .into_iter()
	    .filter(|p| {
		!self.is_prim_clipped(p)
	    }).collect();
        
        for prim in &clipped {
            match *prim {
                Primitive::Point { point } => {
                    let p = &point;
                    rp.points.push(Vec2::new(p.x, p.y));
                }
                Primitive::Line { points } => {
                    let p0 = &points[0];
                    let p1 = &points[1];
                    rp.lines.push(RenderLine::new(
                        &Vec2::new(p0.x, p0.y),
                        &Vec2::new(p1.x, p1.y),
                        EdgeType::Visible,
                    ));
                }
                Primitive::Triangle {
                    ref points,
                    ref edges,
                } => {

                    let points_2d: Vec<_> = points
                        .iter()
                        .map(|p| p.xy())
                        .collect();
                    let n: usize = points_2d.len();

		    // Remove denegerate and counter-clockwise triangles
		    let p01 = points_2d[1] - points_2d[0];
                    let p12 = points_2d[2] - points_2d[1];
                    if (p01.x * p12.y - p01.y * p12.x) <= 1e-4 * p01.norm() * p12.norm() {
			continue;
		    }

                    for i in 0..n {
                        let p0 = vec2(points_2d[i].x, points_2d[i].y);
                        let p1 = vec2(points_2d[(i + 1) % n].x, points_2d[(i + 1) % n].y);

                        rp.lines.push(RenderLine::new(
                            &p0,
                            &p1,
                            edges[i],
                        ));
                    }
                }
            }
        }
        rp
    }
}
