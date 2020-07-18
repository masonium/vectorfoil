use na::{Matrix4, Vector2, Vector3};
use nalgebra as na;
//use nalgebra_glm as glm;

type V2 = Vector2<f32>;
type V3 = Vector3<f32>;

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    Visible,
    Invisible,
    Hidden,
    Clipped,
    Culled,
}

#[derive(Clone)]
pub enum Primitive {
    /// Convex polygon
    Polygon {
        points: Vec<V3>,
        edges: Vec<EdgeType>,
    },
    /// Line segment
    Line {
        points: [V3; 2],
    },

    /// Single point
    Point {
        point: V3,
    },
}

pub struct Renderer {
    clip: Matrix4<f32>,
    cull: bool,
    input_primitives: Vec<Primitive>,
}

/// Output line from the `Renderer`.
#[derive(Debug, Clone, Copy)]
pub struct RenderLine {
    points: [V2; 2],
    edge: EdgeType,
}

impl RenderLine {
    pub fn new(p0: &V2, p1: &V2, e: EdgeType) -> RenderLine {
        RenderLine {
            points: [*p0, *p1],
            edge: e,
        }
    }
}

/// Rendering output from the `Renderer`.
#[derive(Debug, Clone, Default)]
pub struct RenderPaths {
    pub points: Vec<V2>,
    pub hidden_points: Vec<V2>,

    pub lines: Vec<RenderLine>,
}

impl Renderer {
    pub fn new(c: &Matrix4<f32>) -> Renderer {
        Renderer {
            clip: *c,
            cull: false,
            input_primitives: vec![],
        }
    }

    pub fn cull(self, do_cull: bool) -> Renderer {
        Renderer {
            cull: do_cull,
            ..self
        }
    }

    /// add a primitive to the render list
    pub fn add_prim(&mut self, p: Primitive) {
        self.input_primitives.push(p);
    }

    fn proj(&self, p: &V3) -> V3 {
        let r = self.clip * na::Vector4::new(p.x, p.y, p.z, 1.0);
        V3::new(r.x, r.y, r.z) / r.w
    }

    pub fn render(&self) -> RenderPaths {
        let mut rp = RenderPaths::default();
        // multiply, project, no clipping
        for prim in &self.input_primitives {
            match *prim {
                Primitive::Point { point } => {
                    let p = self.proj(&point);
                    rp.points.push(V2::new(p.x, p.y));
                }
                Primitive::Line { points } => {
                    let p0 = self.proj(&points[0]);
                    let p1 = self.proj(&points[1]);
                    rp.lines.push(RenderLine::new(
                        &V2::new(p0.x, p0.y),
                        &V2::new(p1.x, p1.y),
                        EdgeType::Visible,
                    ));
                }
                Primitive::Polygon {
                    ref points,
                    ref edges,
                } => {
                    // less than three points doesn't count.
                    if points.len() < 3 {
                        continue;
                    }
                    let points_2d: Vec<_> = points
                        .iter()
                        .map(|p| self.proj(p))
                        .map(|p| V2::new(p.x, p.y))
                        .collect();
                    let n: usize = points_2d.len();

                    // check culling
                    let culled = if self.cull {
                        let p01 = points_2d[1] - points_2d[0];
                        let p12 = points_2d[2] - points_2d[1];
                        (p01.x * p12.y - p01.y * p12.x) < 0.0
                    } else {
                        false
                    };

                    for i in 0..n {
                        let p0 = V2::new(points_2d[i].x, points_2d[i].y);
                        let p1 = V2::new(points_2d[(i + 1) % n].x, points_2d[(i + 1) % n].y);

                        rp.lines.push(RenderLine::new(&p0, &p1, if culled { EdgeType::Culled } else { edges[i] }));
                    }
                }
            }
        }

        rp
    }
}
