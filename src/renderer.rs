use na::Matrix4;

use crate::common::*;
use crate::intersect::{split_triangle_by_segment, SplitResult};
use crate::primitive::*;
use std::collections::binary_heap::BinaryHeap;
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
    depth_range: [f32; 2],
}

trait VDebug {
    fn na_dbg(&self) -> String;
}

impl VDebug for Vec2 {
    fn na_dbg(&self) -> String {
	format!("[{:.4}, {:.4}]", self.x, self.y)
    }
}
impl VDebug for Vec4 {
    fn na_dbg(&self) -> String {
	format!("[{:.4}, {:.4}, {:.4}]", self.x, self.y, self.z)
    }
}
impl VDebug for Tri {
    fn na_dbg(&self) -> String {
	format!("[0:{}, 1:{}, 2:{}]", self.p[0].xy().na_dbg(), self.p[1].xy().na_dbg(), self.p[2].xy().na_dbg())
    }
}

impl Renderer {
    pub fn new(c: &Matrix4<f32>) -> Renderer {
        Renderer {
            clip: *c,
            input_primitives: vec![],
            depth_range: [-1.0, 1.0],
        }
    }

    /// add a primitive to the render list
    pub fn add_prim(&mut self, p: Primitive) {
        self.input_primitives.push(p);
    }

    /// Add a point to the list, given the primitive.
    pub fn add_point(&mut self, p: Vec3) {
        self.add_prim(Primitive::Point {
            point: p.push(1.0),
        });
    }

    pub fn add_tri(&mut self, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
	self.add_prim(Primitive::Triangle {
	    tri: Tri { p: [p0.push(1.0), p1.push(1.0), p2.push(1.0)], e: [EdgeType::Visible; 3] }
	});
    }

    /// Project the primitive as a whole.
    fn proj_prim(&self, prim: &Primitive) -> Primitive {
        match prim {
            Primitive::Point { point } => Primitive::Point {
                point: self.proj(&point),
            },
            Primitive::Line { points } => Primitive::Line {
                points: [self.proj(&points[0]), self.proj(&points[1])],
            },
            Primitive::Triangle { tri: Tri { p, e } } => Primitive::Triangle {
                tri: Tri {
                    p: [self.proj(&p[0]), self.proj(&p[1]), self.proj(&p[2])],
                    e: e.clone(),
                },
            },
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
            Primitive::Point { point } => self.points_clipped(&[*point]),
            Primitive::Line { points } => self.points_clipped(points),
            Primitive::Triangle { tri: Tri { p, .. } } => self.points_clipped(p),
        }
    }

    /// Return true iff the set of input `points` can be
    /// conservatively clipped because they are all on the wrong side
    /// of a single clipping plane.
    fn points_clipped(&self, points: &[Vec4]) -> bool {
        // Check against each of the 6 clipping planes. If the set of
        // is on the wrong side of any plane, the primitive
        points.iter().all(|v| v.x < -1.0)
            || points.iter().all(|v| v.x > 1.0)
            || points.iter().all(|v| v.y < -1.0)
            || points.iter().all(|v| v.y > 1.0)
            || points.iter().all(|v| v.z < self.depth_range[0])
            || points.iter().all(|v| v.z > self.depth_range[1])
    }

    pub fn render(&self) -> RenderPaths {
        let mut rp = RenderPaths::default();

        // project the primitives
        let projected: Vec<_> = self
            .input_primitives
            .iter()
            .map(|p| self.proj_prim(p))
            .collect();

        let clipped: Vec<_> = projected
            .into_iter()
            .filter(|p| !self.is_prim_clipped(p))
            .collect();

        let mut prim_heap: BinaryHeap<ZsortPrim> = BinaryHeap::with_capacity(clipped.len());
        clipped
            .iter()
            .for_each(|p| prim_heap.push(p.clone().into()));

        'prim_loop: while let Some(x) = prim_heap.pop() {
            let prim = x.p;
            match prim {
                // TODO: For now, points and lines are rendered unconditionally.
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
                Primitive::Triangle { tri } => {
                    let points_2d: Vec<_> = tri.p.iter().map(|p| p.xy()).collect();
                    let n: usize = points_2d.len();

                    // Remove denegerate and counter-clockwise triangles
                    let p01 = points_2d[1] - points_2d[0];
                    let p12 = points_2d[2] - points_2d[1];
                    if (p01.x * p12.y - p01.y * p12.x) <= 1e-4 * p01.norm() * p12.norm() {
                        continue;
                    }

                    // create an intersection
                    // go through every previously-rendered line
                    for l in &rp.lines {
			println!("Testing {} against {} -> {}", 
				 tri.na_dbg(), 
				 l.points[0].na_dbg(), 
				 l.points[1].na_dbg());
                        // try to split the triangle on the line
                        if let SplitResult::Split(tris) =
                            split_triangle_by_segment(&tri, l.points[0], l.points[1])
                        {
                            println!("Splitting into {}:", tris.len());

                            for t in tris {
				println!("    {}", t.na_dbg());
                                prim_heap.push(Primitive::Triangle { tri: t }.into())
                            }
                            continue 'prim_loop;
                        }
                    }

                    // split the triangle

                    for i in 0..n {
                        let p0 = vec2(points_2d[i].x, points_2d[i].y);
                        let p1 = vec2(points_2d[(i + 1) % n].x, points_2d[(i + 1) % n].y);

                        rp.lines.push(RenderLine::new(&p0, &p1, tri.e[i]));
                    }
                }
            }
        }
        rp
    }
}
