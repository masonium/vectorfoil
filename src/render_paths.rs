use crate::common::*;
use crate::primitive::{Tri, ZsortPrim};

/// Output line from the `Renderer`.
#[derive(Debug, Clone, Copy)]
pub struct RenderLine {
    points: [DVec2; 2],
    edge: EdgeType,
}

impl RenderLine {
    pub fn new(p0: DVec2, p1: DVec2, e: EdgeType) -> RenderLine {
        RenderLine {
            points: [p0, p1],
            edge: e,
        }
    }
}

/// Rendering output from the `Renderer`.
#[derive(Debug, Clone, Default)]
pub struct RenderPaths {
    pub points: Vec<DVec2>,
    pub hidden_points: Vec<DVec2>,

    pub lines: Vec<RenderLine>,
}

impl RenderPaths {
    /// Return true iff there are no pieces to render, visible or
    /// hidden.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && self.hidden_points.is_empty() && self.lines.is_empty()
    }

    /// Return a copy of this object with only the visible lines and
    /// points included.
    pub fn visible_only(self) -> RenderPaths {
        RenderPaths {
            points: self.points,
            hidden_points: vec![],

            lines: self
                .lines
                .into_iter()
                .filter(|rl| rl.edge == EdgeType::Visible)
                .collect(),
        }
    }

    pub fn as_svg(&self) -> svg::node::element::Group {
        let mut g = svg::node::element::Group::new();

        for line in &self.lines {
            g = g.add(
                svg::node::element::Line::new()
                    .set("x1", line.points[0].x)
                    .set("y1", line.points[0].y)
                    .set("x2", line.points[1].x)
                    .set("y2", line.points[1].y)
                    .set(
                        "class",
                        match line.edge {
                            EdgeType::Visible => "visible",
                            EdgeType::Invisible => "invisible",
                            EdgeType::Hidden => "hidden",
                            EdgeType::Split => "split",
                            EdgeType::Culled => "culled",
                        },
                    ),
            );
        }

        g
    }
}

impl<'a> std::iter::FromIterator<&'a ZsortPrim> for RenderPaths {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a ZsortPrim>,
    {
        let mut rp = RenderPaths::default();
        for prim in iter {
            match prim.p {
                Primitive::Point { point } => {
                    rp.points.push(point.xy());
                }
                Primitive::Line { points } => {
                    rp.lines.push(RenderLine::new(
                        points[0].xy(),
                        points[1].xy(),
                        EdgeType::Visible,
                    ));
                }
                Primitive::Triangle { tri: Tri { p, e } } => {
                    rp.lines.push(RenderLine::new(p[0].xy(), p[1].xy(), e[0]));
                    rp.lines.push(RenderLine::new(p[1].xy(), p[2].xy(), e[1]));
                    rp.lines.push(RenderLine::new(p[2].xy(), p[0].xy(), e[2]));
                }
            }
        }
        rp
    }
}
