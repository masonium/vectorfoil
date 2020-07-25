//! `RenderPaths` are the output of the vectorfoil `Renderer`.
//!
//! The structure contains both the direct visible primitive lines and
//! points, as well as some of the points that have been filtered out
//! during the rendering process.

use crate::common::*;
use crate::primitive::{Tri, ZsortPrim};
use std::collections::HashMap;
use svg::Document;
use svg::node::element::{self, Group, Style};

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

pub struct SvgOptions {
    pub width: f64,
    pub height: f64,
    pub by_layer: bool,
}

/// Rendering output from the `Renderer`.
#[derive(Debug, Clone, Default)]
pub struct RenderPaths {
    pub points: Vec<DVec2>,

    pub lines: Vec<RenderLine>,
}

impl RenderPaths {
    /// Return true iff there are no pieces to render, visible or
    /// hidden.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && self.lines.is_empty()
    }

    /// Return a copy of this object with only the visible lines and
    /// points included.
    pub fn visible_only(self) -> RenderPaths {
        RenderPaths {
            points: self.points,

            lines: self
                .lines
                .into_iter()
                .filter(|rl| rl.edge == EdgeType::Visible)
                .collect(),
        }
    }

    /// Render a complete svg document from this structure.
    pub fn as_standalone_svg(self: &RenderPaths, options: &SvgOptions) -> Document {
        let half_width = options.width * 0.5;
        let half_height = options.height * 0.5;
        let mut d = Document::new()
            .set("width", format!("{}", half_width * 2.0))
            .set("height", format!("{}", half_height * 2.0))
            .add(Style::new(
                ".visible { stroke-width: 0.005; fill: none; stroke: #444444; }
.hidden { stroke-width: 0.002; fill: none; stroke: #2222cc; stroke-dasharray: 0.01 0.005; }
.invisible { stroke-width: 0.001; fill: none; stroke: #aaaaaa; stroke-dasharray: 0.001 0.001; }
.split { stroke-width: 0.001; fill: none; stroke: #22cc22; stroke-dasharray: 0.002 0.002; }
.culled { stroke-width: 0.001; fill: none; stroke: #cc2222; stroke-dasharray: 0.005 0.005; }",
            ));

        let mut g = Group::new().set(
            "transform",
            format!(
                "translate({} {}) scale({} -{})",
                half_width, half_height, half_width, half_height
            ),
        );

        g = g.add(self.as_svg_group(options));
        d = d.add(g);

        d
    }

    fn add_line(group: Group, line: &RenderLine, class: Option<&str>) -> Group {
	let mut line = element::Line::new()
                    .set("x1", line.points[0].x)
                    .set("y1", line.points[0].y)
                    .set("x2", line.points[1].x)
                    .set("y2", line.points[1].y);
	if let Some(class_name) = class {
	    line = line.set("class", class_name);
	}
	group.add(line)
    }

    pub fn as_svg_group(&self, options: &SvgOptions) -> Group {
        let mut g = Group::new();
	let mut lines_by_type: HashMap<EdgeType, Vec<&RenderLine>> = HashMap::new();

	if options.by_layer {
	    // Group the lines by edge type, and render each group. 
	    for line in &self.lines {
		lines_by_type.entry(line.edge).or_default().push(&line);
	    }
	    
	    for (edge_type, lines) in lines_by_type {
		let mut group = Group::new().set("class", edge_type.class_name());
		for line in &lines {
		    group = Self::add_line(group, line, None);
		}
		g = g.add(group);
	    }
	} else {
            for line in &self.lines {
		g = Self::add_line(g, line, Some(line.edge.class_name()));
            }
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
