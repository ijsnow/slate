use super::operation::Operation;
use super::path::{self, Path};
use super::point::Point;

pub enum Affinity {
    Forward,
    Backward,
    Outward,
    Inward,
    None,
}

impl Default for Affinity {
    fn default() -> Self {
        Self::Inward
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    anchor: Point,
    focus: Point,
}

impl Range {
    pub fn new(anchor: Point, focus: Point) -> Self {
        Self { anchor, focus }
    }

    pub fn edges(&self, reverse: bool) -> (&Point, &Point) {
        if self.is_backward() == reverse {
            (&self.anchor, &self.focus)
        } else {
            (&self.focus, &self.anchor)
        }
    }

    fn includes_path(&self, target: &Path) -> bool {
        let (start, end) = self.edges(false);
        let is_after_start = target >= &start.path;
        let is_before_end = target <= &end.path;
        is_after_start && is_before_end
    }

    fn includes_point(&self, target: &Point) -> bool {
        let (start, end) = self.edges(false);
        let is_after_start = target >= start;
        let is_before_end = target <= end;
        is_after_start && is_before_end
    }

    fn includes_range(&self, target: &Range) -> bool {
        if self.includes_point(&target.anchor) || self.includes_point(&target.focus) {
            return true;
        }
        let (rs, re) = self.edges(false);
        let (ts, te) = target.edges(false);
        rs.is_before(ts) && re.is_before(te)
    }

    fn intersection(&self, another: &Range) -> Option<Range> {
        let (s1, e1) = self.edges(false);
        let (s2, e2) = self.edges(false);
        let start = if s1.is_before(s2) { s2 } else { s1 };
        let end = if e1.is_before(e2) { e1 } else { e2 };
        if end.is_before(start) {
            None
        } else {
            Some(Range {
                anchor: start.clone(),
                focus: end.clone(),
            })
        }
    }

    fn is_backward(&self) -> bool {
        self.anchor.is_after(&self.focus)
    }

    fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    fn is_expanded(&self) -> bool {
        !self.is_collapsed()
    }

    fn is_forward(&self) -> bool {
        !self.is_backward()
    }

    fn start(&self) -> Point {
        let (s, _) = self.edges(false);
        s.clone()
    }

    fn transform(range: &Range, op: &Operation, affinity: Affinity) -> Option<Range> {
        let (affinityAnchor, affinityFocus): (path::Affinity, path::Affinity) = match affinity {
            Affinity::Inward => {
                if range.is_forward() {
                    (path::Affinity::Forward, path::Affinity::Backward)
                } else {
                    (path::Affinity::Backward, path::Affinity::Forward)
                }
            }
            Affinity::Outward => {
                if range.is_forward() {
                    (path::Affinity::Backward, path::Affinity::Forward)
                } else {
                    (path::Affinity::Forward, path::Affinity::Backward)
                }
            }
            Affinity::Forward => (path::Affinity::Forward, path::Affinity::Forward),
            Affinity::Backward => (path::Affinity::Backward, path::Affinity::Backward),
            Affinity::None => (path::Affinity::None, path::Affinity::None),
        };

        let range = range.clone();

        let anchor = Point::transform(&range.anchor, op, affinityAnchor);
        let focus = Point::transform(&range.anchor, op, affinityFocus);

        if anchor.is_none() || focus.is_none() {
            None
        } else {
            Some(Range {
                anchor: anchor.unwrap(),
                focus: focus.unwrap(),
            })
        }
    }

    fn points(&self) -> (&Point, &Point) {
        (&self.anchor, &self.focus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_collapsed() {
        let input = Range {
            anchor: Point {
                path: Path::new(vec![0]),
                offset: 0,
            },
            focus: Point {
                path: Path::new(vec![0]),
                offset: 0,
            },
        };
        assert_eq!(
            input.edges(false),
            (
                &Point {
                    path: Path::new(vec![0]),
                    offset: 0,
                },
                &Point {
                    path: Path::new(vec![0]),
                    offset: 0,
                }
            )
        );
    }

    #[test]
    fn includes_path_after() {
        let input = Range {
            anchor: Point {
                path: Path::new(vec![1]),
                offset: 0,
            },
            focus: Point {
                path: Path::new(vec![3]),
                offset: 0,
            },
        };
        let target = vec![4].into();
        assert!(!input.includes_path(&target));
    }

    #[test]
    fn includes_path_before() {
        let input = Range {
            anchor: Point {
                path: Path::new(vec![1]),
                offset: 0,
            },
            focus: Point {
                path: Path::new(vec![3]),
                offset: 0,
            },
        };
        let target = vec![0].into();
        assert!(!input.includes_path(&target));
    }

    #[test]
    fn includes_path_end() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = vec![3].into();
        assert!(input.includes_path(&target));
    }

    #[test]
    fn includes_path_inside() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = vec![2].into();
        assert!(input.includes_path(&target));
    }

    #[test]
    fn includes_path_start() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = vec![1].into();
        assert!(input.includes_path(&target));
    }

    #[test]
    fn includes_point_inside() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = Point {
            path: vec![2].into(),
            offset: 0,
        };
        assert!(input.includes_point(&target));
    }

    #[test]
    fn includes_point_offset_before() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 3,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = Point {
            path: vec![1].into(),
            offset: 0,
        };
        assert!(!input.includes_point(&target));
    }

    #[test]
    fn includes_point_path_after() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 3,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = Point {
            path: vec![4].into(),
            offset: 0,
        };
        assert!(!input.includes_point(&target));
    }

    #[test]
    fn includes_point_path_before() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = Point {
            path: vec![0].into(),
            offset: 0,
        };
        assert!(!input.includes_point(&target));
    }

    #[test]
    fn includes_point_start() {
        let input = Range {
            anchor: Point {
                path: vec![1].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        let target = Point {
            path: vec![1].into(),
            offset: 0,
        };
        assert!(input.includes_point(&target));
    }

    #[test]
    fn is_backward_backward() {
        let input = Range {
            anchor: Point {
                path: vec![3].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![0].into(),
                offset: 0,
            },
        };
        assert!(input.is_backward());
    }

    #[test]
    fn is_backward_collapsed() {
        let input = Range {
            anchor: Point {
                path: vec![0].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![0].into(),
                offset: 0,
            },
        };
        assert!(!input.is_backward());
    }

    #[test]
    fn is_backward_forward() {
        let input = Range {
            anchor: Point {
                path: vec![0].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        assert!(!input.is_backward());
    }

    #[test]
    fn is_collapsed_collapsed() {
        let input = Range {
            anchor: Point {
                path: vec![0].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![0].into(),
                offset: 0,
            },
        };
        assert!(input.is_collapsed());
    }

    #[test]
    fn is_collapsed_expanded() {
        let input = Range {
            anchor: Point {
                path: vec![0].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        assert!(!input.is_collapsed());
    }

    #[test]
    fn points_full_selection() {
        let input = Range {
            anchor: Point {
                path: vec![0].into(),
                offset: 0,
            },
            focus: Point {
                path: vec![3].into(),
                offset: 0,
            },
        };
        assert_eq!(input.points(), (&input.anchor, &input.focus));
    }
}
