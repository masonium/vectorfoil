use na::Matrix4;

use crate::common::*;
use crate::intersect::{split_triangle_by_segment, triangle_in_triangle_2d, SplitResult};
use crate::primitive::*;
use crate::render_paths::RenderPaths;
use std::collections::binary_heap::BinaryHeap;
//use nalgebra_glm as glm;

pub struct Renderer {
    clip: Matrix4<f64>,
    input_primitives: Vec<Primitive>,
    depth_range: [f64; 2],
}

trait VDebug {
    fn na_dbg(&self) -> String;
}

impl VDebug for DVec2 {
    fn na_dbg(&self) -> String {
	format!("[{:.4}, {:.4}]", self.x, self.y)
    }
}
impl VDebug for DVec4 {
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
    pub fn new(c: &Matrix4<f64>) -> Renderer {
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
    pub fn add_point(&mut self, p: DVec3) {
        self.add_prim(Primitive::Point { point: p.push(1.0) });
    }

    /// Add a triangle to the renderer, with all visible edges.
    pub fn add_triangle(&mut self, p0: &DVec3, p1: &DVec3, p2: &DVec3) {
        self.add_prim(Primitive::Triangle {
            tri: Tri {
                p: [p0.push(1.0), p1.push(1.0), p2.push(1.0)],
                e: [EdgeType::Visible; 3],
            },
        });
    }

    /// Add a polygon to the list, with all outside edges visible.
    ///
    /// # Remarks
    ///
    /// Polygons are internally translated into a triangles as a
    /// triangle fan. The inner edges are marked as Invisible.
    pub fn add_polygon(&mut self, p: &[DVec3]) {
        for i in 0..p.len() - 2 {
            let e0 = if i == 0 {
                EdgeType::Visible
            } else {
                EdgeType::Invisible
            };
            let e2 = if i == p.len() - 3 {
                EdgeType::Visible
            } else {
                EdgeType::Invisible
            };
            self.add_prim(Primitive::Triangle {
                tri: Tri {
                    p: [p[0].push(1.0), p[i + 1].push(1.0), p[i + 2].push(1.0)],
                    e: [e0, EdgeType::Visible, e2],
                },
            });
        }
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
    fn proj(&self, p: &DVec4) -> DVec4 {
        let r = self.clip * p;
        vec4(r.x / r.w, r.y / r.w, r.z / r.w, r.w)
    }

    /// Return true if a (projected) primitve can be trivially culled
    /// from the viewing frustum.
    fn is_prim_culled(&self, prim: &Primitive) -> bool {
        match prim {
            Primitive::Point { point } => self.points_culled(&[*point]),
            Primitive::Line { points } => self.points_culled(points),
            Primitive::Triangle { tri: Tri { p, .. } } => self.points_culled(p),
        }
    }

    /// Return true iff the set of input `points` can be
    /// conservatively culled because they are all on the wrong side
    /// of a single frustum plane.
    fn points_culled(&self, points: &[DVec4]) -> bool {
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
        let culled: Vec<_> = self
            .input_primitives
            .iter()
	// project the primitives into clip space
            .map(|p| self.proj_prim(p))
	// (conservatively) cull the primitives that are
	// completely outside of the render region.
            .filter(|p| !self.is_prim_culled(p))
            .collect();

        let mut prim_heap: BinaryHeap<ZsortPrim> = culled
            .iter()
            .map(|p| p.clone().into())
            .collect();

	// Tentatively rendered primitives (that might be later rejected.
	let mut rendered_prims = vec![];

        'prim_loop: while let Some(mut x) = prim_heap.pop() {
            let prim = &mut x.p;
            match prim {
                // TODO: For now, points and lines are rendered unconditionally.
                Primitive::Point { .. } => {
		    rendered_prims.push(x);
                }
                Primitive::Line { .. } => {
		    rendered_prims.push(x);
                }
                Primitive::Triangle { ref mut tri } => {
                    // Remove denegerate and counter-clockwise triangles
                    let p01 = tri.p[1].xy() - tri.p[0].xy();
                    let p12 = tri.p[2].xy() - tri.p[1].xy();
                    if (p01.x * p12.y - p01.y * p12.x) <= 1e-4 * p01.norm() * p12.norm() {
                        continue;
                    }

                    // Go through every previously-rendered triangle, and try to intersect it with
		    // every line segment (implied from previous triangles).
                    for zp in &rendered_prims {
			if let Primitive::Triangle { tri: test_tri } = &zp.p {
			    // ignore hidden triangles
			    if test_tri.is_hidden() {
				continue;
			    }
			    for i in 0..3 {

				println!("Testing triangle {} against ({} {}):",
					 tri.na_dbg(), 
					 test_tri.p[i].xy().na_dbg(), 
					 test_tri.p[(i+1)%3].xy().na_dbg());
				
				// try to split the triangle on the line
				if let SplitResult::Split(tris) =
				    split_triangle_by_segment(&tri, test_tri.p[i].xy(), test_tri.p[(i+1)%3].xy())
				{
				     println!("Splitting into {}:", tris.len());

				    for t in tris {
					println!("    {}", t.na_dbg());
					prim_heap.push(Primitive::Triangle { tri: t }.into())
				    }
				    continue 'prim_loop;
				}
			    }

			    // Check if the new triangle is contained
			    // within the current triangle.
			    if triangle_in_triangle_2d(&tri, &test_tri) {
				// For now, we assume that the new tri is behind.
				tri.hide(); 
			    }

			}
		    }

		    // Here, we can tentatively render the
		    // primitive. (We might reject it later.)
		    rendered_prims.push(x);
                }
	    }
        }

        rendered_prims.iter().collect()
    }
}
