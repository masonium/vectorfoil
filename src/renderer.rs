use na::Matrix4;

use crate::common::*;
use crate::intersect::{split_triangle_by_segment, triangle_in_triangle_2d, SplitResult};
use crate::primitive::*;
use crate::render_paths::RenderPaths;
use itertools::Itertools;
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
        format!("vec2({:.10}, {:.10})", self.x, self.y)
    }
}
impl VDebug for DVec4 {
    fn na_dbg(&self) -> String {
        format!("[{:.10}, {:.10}, {:.10}]", self.x, self.y, self.z)
    }
}
impl VDebug for Tri {
    fn na_dbg(&self) -> String {
        format!(
            "[vec4({}), vec4({}), vec4({})]",
            self.p[0].xy().na_dbg(),
            self.p[1].xy().na_dbg(),
            self.p[2].xy().na_dbg()
        )
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

    pub fn add_line(&mut self, p0: DVec3, p1: DVec3) {
        self.add_prim(Primitive::Line { points: [p0.push(1.0), p1.push(1.0)] });
    }

    /// Add a triangle to the renderer, with all visible edges.
    pub fn add_triangle(&mut self, p0: DVec3, p1: DVec3, p2: DVec3) {
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
            .filter_map(|p| {
                if let Primitive::Triangle { mut tri } = p.clone() {
                    let winding = tri.winding_2d();
                    match winding {
                        Winding::Clockwise => {
                            tri.reverse();
                            Some(Primitive::Triangle { tri })
                        }
                        Winding::Degenerate => None,
                        _ => Some(p),
                    }
                } else {
                    Some(p)
                }
            })
            .collect();

        let mut prim_heap: BinaryHeap<ZsortPrim> =
            culled.iter().map(|p| p.clone().into()).collect();

        // Tentatively rendered primitives (that might be later rejected.
        let mut rendered_prims = vec![];

        let mut iter = 0;
        render_partial(
            None,
            &rendered_prims,
            &prim_heap,
            None,
            None,
            &mut iter,
            false,
        );

        'prim_loop: while let Some(mut x) = prim_heap.pop() {
            let prim = &x.p;
            let mut hidden = false;
            let mut added = false;
            match prim {
                // TODO: For now, points and lines are rendered unconditionally.
                Primitive::Point { .. } => {
                    rendered_prims.push(x);
                }
                Primitive::Line { .. } => {
                    rendered_prims.push(x);
                }
                Primitive::Triangle { ref tri } => {
                    if tri.winding_2d() == Winding::Degenerate {
                        continue;
                    }

                    render_partial(
                        &*tri,
                        &rendered_prims,
                        &prim_heap,
                        None,
                        None,
                        &mut iter,
                        false,
                    );

                    // Go through every previously-rendered triangle, and try to intersect it with
                    // every line segment (implied from previous triangles).
                    for (izp, zp) in rendered_prims.iter().enumerate() {
                        if let Primitive::Triangle { tri: test_tri } = &zp.p {
                            // ignore hidden triangles
                            if test_tri.is_hidden() {
                                continue;
                            }

                            for i in 0..3 {
                                if x.already_checked(izp, i) {
                                    println!("{} skipping ({} {})", iter, izp, i);
                                    continue;
                                }

                                let pa = test_tri.p[i].xy();
                                let pb = test_tri.p[(i + 1) % 3].xy();
                                println!(
                                    "{} Testing triangle {} against ({} {}):",
                                    iter,
                                    tri.na_dbg(),
                                    pa.na_dbg(),
                                    pb.na_dbg()
                                );

                                // try to split the triangle on the line
                                if let SplitResult::Split(tris) =
                                    split_triangle_by_segment(&tri, pa, pb)
                                {
                                    println!("{} Splitting into {}:", iter, tris.len());
                                    render_partial(
                                        None,
                                        &rendered_prims,
                                        &prim_heap,
                                        (&tris, (pa, pb)),
                                        None,
                                        &mut iter,
                                        added,
                                    );

                                    let mut new_hs = x.presplit.clone();
                                    new_hs.insert((izp, i));
                                    for t in tris {
                                        println!("{}    {}", iter, t.na_dbg());
                                        prim_heap.push(ZsortPrim::new(
                                            Primitive::Triangle { tri: t },
                                            &new_hs,
                                        ));
                                    }
                                    continue 'prim_loop;
                                }
                            }

                            // Check if the new triangle is contained
                            // within the current triangle.
                            println!(
                                "{} Testing triangle {} inside {}",
                                iter,
                                tri.na_dbg(),
                                test_tri.na_dbg()
                            );
                            if triangle_in_triangle_2d(&tri, &test_tri) {
                                // For now, we assume that the new tri is behind.
                                println!(
                                    "{} Hiding triangle {} inside {}",
                                    iter,
                                    tri.na_dbg(),
                                    test_tri.na_dbg()
                                );
                                hidden = true;
                                render_partial(
                                    None,
                                    &rendered_prims,
                                    &prim_heap,
                                    None,
                                    &*tri,
                                    &mut iter,
                                    added,
                                );
                                break;
                            }
                        }
                    }

                    if hidden {
                        x.p.hide();
                    }

                    // Here, we can tentatively render the
                    // primitive. (We might reject it later.)
                    rendered_prims.push(x);
                    added = true;
                }
            }

            render_partial(
                None,
                &rendered_prims,
                &prim_heap,
                None,
                None,
                &mut iter,
                added,
            );
        }

        rendered_prims.iter().collect()
    }
}

fn render_partial<'a, 'b, 'c>(
    next: impl Into<Option<&'b Tri>>,
    rendered: &Vec<ZsortPrim>,
    heap: &BinaryHeap<ZsortPrim>,
    split: impl Into<Option<(&'c Vec<Tri>, (DVec2, DVec2))>>,
    hidden: impl Into<Option<&'a Tri>>,
    iter: &mut usize,
    added: bool,
) {
    // add the all of the triangles, highlighting the current one.
    let dpi = 72.0;
    let width = 10.0;
    let height = 10.0;
    let mut d = svg::Document::new()
        .set("width", format!("{}", width * dpi))
        .set("height", format!("{}", height * dpi))
        .add(svg::node::element::Style::new(
            ".rendered { stroke-width: 0.005; fill: none; stroke: #444444; }
.latest { stroke-width: 0.005; fill: #00cc00; opacity: 0.5; stroke: #444444; }
.next { stroke-width: 0.002; fill: #0000cc; opacity: 0.5; stroke: #00cc00 ; }
.split { stroke-width: 0.002; fill: #cc0000; opacity: 0.5; stroke: #666666 ; }
.hidden { stroke-width: 0.002; fill: #000000; opacity: 0.5; stroke: #666666; }
.ready { stroke-width: 0.002; fill: none; stroke: #999999; stroke-dasharray: 0.01 0.01; }
.split { stroke-width: 0.002; fill: none; stroke: #000000; },
.split_line { stroke-width: 0.02; stroke: #000000; stroke-dasharray: 0.004 0.004 }",
        ));

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

    d = d.add(
        svg::node::element::Rectangle::new()
            .set("width", width * dpi)
            .set("height", height * dpi)
            .set("style", "fill: #ffffff"),
    );

    let add_prim = |tri: &Tri, class: &str, g: svg::node::element::Group| {
        g.add(
            svg::node::element::Polygon::new()
                .set(
                    "points",
                    tri.p.iter().map(|p| format!("{},{}", p[0], p[1])).join(" "),
                )
                .set("class", class),
        )
    };
    let add_color_prim = |tri: &Tri, class: &str, color: &str, g: svg::node::element::Group| {
        g.add(
            svg::node::element::Polygon::new()
                .set(
                    "points",
                    tri.p.iter().map(|p| format!("{},{}", p[0], p[1])).join(" "),
                )
                .set("class", class)
                .set("style", format!("fill: {};", color)),
        )
    };
    let add_line = |p0: &DVec2, p1: &DVec2, g: svg::node::element::Group| {
        g.add(
            svg::node::element::Polyline::new()
                .set("points", format!("{},{} {},{}", p0.x, p0.y, p1.x, p1.y))
                .set("class", "split_line"),
        )
    };

    for (i, zprim) in rendered.iter().enumerate() {
        match zprim.p {
            Primitive::Triangle { ref tri } => {
                g = add_prim(
                    tri,
                    if tri.is_hidden() {
                        "hidden"
                    } else if added && i == rendered.len() - 1 {
                        "latest"
                    } else {
                        "rendered"
                    },
                    g,
                );
            }
            _ => {}
        }
    }

    for zprim in heap.iter() {
        match zprim.p {
            Primitive::Triangle { ref tri } => {
                g = add_prim(tri, "ready", g);
            }
            _ => {}
        }
    }

    for tri in hidden.into().iter() {
        g = add_prim(tri, "hidden", g)
    }
    for tri in next.into().iter() {
        g = add_prim(tri, "next", g)
    }

    let colors3 = ["#ff0000", "#bb0000", "#880000"];
    let colors2 = ["#ff00ff", "#bb00bb"];

    if let Some((split_tris, split_line)) = split.into() {
        if split_tris.len() == 3 {
            for (tri, color) in split_tris.iter().zip(colors3.iter()) {
                g = add_color_prim(tri, "split", color, g)
            }
        } else {
            for (tri, color) in split_tris.iter().zip(colors2.iter()) {
                g = add_color_prim(tri, "split", color, g)
            }
        }

        g = add_line(&split_line.0, &split_line.1, g);
    }

    d = d.add(g);

    svg::save(format!("x{:06}.svg", iter), &d).ok();
    *iter += 1;
}
