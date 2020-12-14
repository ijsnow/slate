use super::{Path, Point, Range};

pub enum Location {
    Path(Path),
    Point(Point),
    Range(Range),
}

pub struct Span(Path, Path);
