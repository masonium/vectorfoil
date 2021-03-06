use crate::common::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Visible,
    Invisible,
    Hidden, // behind another triangle
    Split,  // generated by an internal split
    Culled,
}

impl EdgeType {
    pub fn class_name(&self) -> &'static str {
	use EdgeType::*;
	match self {
	    Visible => "visible",
	    Invisible => "invisible",
	    Hidden => "hidden",
	    Split => "split",
	    Culled => "culled",
	}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winding {
    Clockwise,
    CounterClockwise,
    Degenerate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tri {
    pub p: [DVec4; 3],
    pub e: [EdgeType; 3],
}

impl Tri {
    /// Hide all edges of the triangle, consuming itself.
    pub fn hide(&mut self) {
        self.e[0] = EdgeType::Hidden;
        self.e[1] = EdgeType::Hidden;
        self.e[2] = EdgeType::Hidden;
    }

    /// Return true iff all of the edges are hidden.
    pub fn is_hidden(&self) -> bool {
        self.e.iter().all(|x| *x == EdgeType::Hidden)
    }

    /// Return the winding of the triangle, assuming a 2d projection.
    pub fn winding_2d(&self) -> Winding {
        let p01 = self.p[1].xy() - self.p[0].xy();
        let p12 = self.p[2].xy() - self.p[1].xy();
        let signed_unscaled_area = p01.x * p12.y - p01.y * p12.x;
        let threshold = EPS * p01.norm() * p12.norm();
        if signed_unscaled_area > threshold {
            Winding::CounterClockwise
        } else if signed_unscaled_area < -threshold {
            Winding::Clockwise
        } else {
            Winding::Degenerate
        }
    }

    /// Reverse the ordering of the vertices and edges
    pub fn reverse(mut self) -> Tri {
        self.p.swap(1, 2);
        self.e.swap(0, 2);
        self
    }

    /// Mark all edges as culled.
    pub fn cull(self) -> Tri {
        Tri {
            p: self.p,
            e: [EdgeType::Culled; 3],
        }
    }

    pub fn is_culled(&self) -> bool {
        self.e.iter().all(|x| *x == EdgeType::Culled)
    }
}

/// Internally, all primitive coordinates are kept in 4D as (x/w, y/w,
/// z/w, w).
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    /// Triangle
    Triangle { tri: Tri },

    /// Line segment
    Line { points: [DVec4; 2] },

    /// Single point
    Point { point: DVec4 },
}

impl Primitive {
    pub fn centroid(&self) -> DVec3 {
        match self {
            Self::Point { point } => point.xyz(),
            Self::Line { points } => (points[0] + points[1]).xyz() * 0.5,
            Self::Triangle { tri: Tri { p, .. } } => { p[0] + p[1] + p[2] }.xyz() / 3.0,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Primitive::Triangle { tri } => tri.is_hidden(),
            _ => false,
        }
    }

    pub fn hide(&mut self) {
        match self {
            Primitive::Triangle { ref mut tri } => tri.hide(),
            _ => {}
        }
    }
}

/// Wrapper for `Primitive` on the heap, to be sorted by z-value for
/// rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct ZsortPrim {
    pub p: Primitive,
    z: f64,
    pub presplit: HashSet<(usize, usize)>,
}

impl From<Primitive> for ZsortPrim {
    fn from(p: Primitive) -> Self {
        let z = -p.centroid().z;
        ZsortPrim {
            p,
            z,
            presplit: HashSet::new(),
        }
    }
}

impl ZsortPrim {
    pub fn new(p: Primitive, hs: &HashSet<(usize, usize)>) -> ZsortPrim {
        let z = -p.centroid().z;
        ZsortPrim {
            p,
            z,
            presplit: hs.clone(),
        }
    }

    pub fn already_checked(&self, izp: usize, i: usize) -> bool {
        self.presplit.contains(&(izp, i))
    }
}

impl Eq for ZsortPrim {}

impl std::cmp::PartialOrd for ZsortPrim {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.z.partial_cmp(&rhs.z)
    }
}

impl std::cmp::Ord for ZsortPrim {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}
