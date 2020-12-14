use std::cmp::{min, Ord, Ordering};

use super::operation::Operation;
use super::path::{Affinity, Path};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Point {
    pub path: Path,
    pub offset: usize,
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.path.cmp(&other.path);
        if result == Ordering::Equal {
            return self.offset.cmp(&other.offset);
        }
        result
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Point {
    pub fn is_after(&self, another: &Point) -> bool {
        self > another
    }

    pub fn is_before(&self, another: &Point) -> bool {
        self < another
    }

    pub fn transform(point: &Point, op: &Operation, affinity: Affinity) -> Option<Point> {
        let Point { path, offset } = point;
        let mut point = point.clone();

        match &op {
            op @ Operation::InsertNode { .. } | op @ Operation::MoveNode { .. } => {
                point.path = Path::transform(path, op, affinity).unwrap();
            }
            Operation::InsertText {
                path: oppath,
                offset: opoffset,
                text,
            } => {
                if oppath == path && opoffset <= offset {
                    point.offset += text.len();
                }
            }
            Operation::MergeNode {
                path: oppath,
                position: oppos,
                ..
            } => {
                if oppath == path {
                    point.offset += oppos;
                }
                point.path = Path::transform(oppath, op, affinity).unwrap();
            }
            Operation::RemoveText {
                path: oppath,
                offset: opoffset,
                text,
                ..
            } => {
                if oppath == path || oppath.is_ancestor(path) {
                    point.offset -= min(opoffset - opoffset, text.len());
                }
                point.path = Path::transform(oppath, op, affinity).unwrap();
            }
            Operation::RemoveNode { path: oppath, .. } => {
                if oppath == path || oppath.is_ancestor(path) {
                    return None;
                }
                point.path = Path::transform(oppath, op, affinity).unwrap();
            }
            Operation::SplitNode {
                path: oppath,
                position: opposition,
                ..
            } => {
                if oppath == path {
                    if opposition == offset && matches!(affinity, Affinity::None) {
                        return None;
                    } else if opposition < offset
                        || (opposition == offset && matches!(affinity, Affinity::Forward))
                    {
                        point.offset -= opposition;

                        point.path = Path::transform(path, op, Affinity::Forward).unwrap();
                    }
                } else {
                    point.path = Path::transform(path, op, affinity).unwrap();
                }
            }
            _ => {}
        }

        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_path_after_offset_after() {
        let a = Point {
            path: Path::new(vec![0, 4]),
            offset: 7,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a > b);
    }

    #[test]
    fn compare_path_after_offset_before() {
        let a = Point {
            path: Path::new(vec![0, 4]),
            offset: 0,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a > b);
    }

    #[test]
    fn compare_path_after_offset_equal() {
        let a = Point {
            path: Path::new(vec![0, 4]),
            offset: 3,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a > b);
    }

    #[test]
    fn compare_path_before_offset_after() {
        let a = Point {
            path: Path::new(vec![0, 0]),
            offset: 4,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 0,
        };
        assert!(a < b);
    }

    #[test]
    fn compare_path_before_offset_before() {
        let a = Point {
            path: Path::new(vec![0, 0]),
            offset: 0,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a < b);
    }

    #[test]
    fn compare_path_before_offset_equal() {
        let a = Point {
            path: Path::new(vec![0, 0]),
            offset: 0,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 0,
        };
        assert!(a < b);
    }

    #[test]
    fn compare_path_equal_offset_after() {
        let a = Point {
            path: Path::new(vec![0, 1]),
            offset: 7,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a > b);
    }

    #[test]
    fn compare_path_equal_offset_before() {
        let a = Point {
            path: Path::new(vec![0, 1]),
            offset: 0,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 3,
        };
        assert!(a < b);
    }

    #[test]
    fn compare_path_equal_offset_equal() {
        let a = Point {
            path: Path::new(vec![0, 1]),
            offset: 7,
        };
        let b = Point {
            path: Path::new(vec![0, 1]),
            offset: 7,
        };
        assert!(a == b);
    }
}
