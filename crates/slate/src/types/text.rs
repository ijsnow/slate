use std::collections::HashSet;

use super::{node::Descendant, Range};

pub type Decoration = (Range, HashSet<String>);

bitflags::bitflags! {
    pub struct Marks: u32 {
        const BOLD = 1 << 1;
        const ITALIC = 1 << 2;
        const UNDERLINE = 1 << 3;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text(String, Marks, HashSet<String>);

impl Into<Descendant> for Text {
    fn into(self) -> Descendant {
        Descendant::Text(self)
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into(), Marks::empty(), HashSet::new())
    }

    pub fn with_meta(text: impl Into<String>, meta: HashSet<String>) -> Self {
        Self(text.into(), Marks::empty(), meta)
    }

    pub fn with_marks(text: impl Into<String>, marks: Marks) -> Self {
        Self(text.into(), marks, HashSet::new())
    }

    /// Check if two Text nodes have the same **marks** (doesn't compare values values).
    pub fn matches(a: Self, b: Self) -> bool {
        a.1.contains(b.1)
    }

    pub fn decorations(self, decorations: Vec<Decoration>) -> Vec<Text> {
        let mut leaves = vec![self];

        for (range, dec) in decorations {
            let (start, end) = range.edges(false);
            let mut next = vec![];
            let mut o = 0;

            for l in leaves.iter() {
                let leaf = l.clone();
                let len = leaf.0.len();
                let offset = o;
                o += leaf.0.len();

                // If the range encompases the entire leaf, add the range.
                if start.offset <= offset && end.offset >= offset + len {
                    next.push(Text(leaf.0, leaf.1, leaf.2.union(&dec).cloned().collect()));
                    continue;
                }

                // If the range starts after the leaf, or ends before it, continue.
                if start.offset > offset + len
                    || end.offset < offset
                    || (end.offset == offset && offset != 0)
                {
                    next.push(leaf);
                    continue;
                }

                // Otherwise we need to split the leaf, at the start, end, or both,
                // and add the range to the middle intersecting section. Do the end
                // split first since we don't need to update the offset that way.
                let mut middle = leaf;
                let mut before: Option<Text> = None;
                let mut after: Option<Text> = None;

                if end.offset < offset + len {
                    let off = end.offset - offset;
                    after = Some(Text(middle.0[off..].into(), middle.1, middle.2.clone()));
                    middle = Text(middle.0[..off].into(), middle.1, middle.2);
                }

                if start.offset > offset {
                    let off = start.offset - offset;
                    before = Some(Text(middle.0[..off].into(), middle.1, middle.2.clone()));
                    middle = Text(middle.0[off..].into(), middle.1, middle.2);
                }

                middle.2 = middle.2.union(&dec).cloned().collect();

                if let Some(b) = before {
                    next.push(b);
                }

                next.push(middle);

                if let Some(a) = after {
                    next.push(a);
                }
            }

            leaves = next;
        }

        leaves
    }
}

#[cfg(test)]
mod tests {
    use super::super::Point;
    use super::*;

    #[test]
    fn matches_empty_true() {
        assert!(Text::matches(
            Text("".into(), Marks::BOLD, vec![].iter().cloned().collect()),
            Text("".into(), Marks::empty(), vec![].iter().cloned().collect())
        ));
    }

    #[test]
    fn matches_false() {
        assert!(!Text::matches(
            Text("".into(), Marks::BOLD, vec![].iter().cloned().collect()),
            Text("".into(), Marks::ITALIC, vec![].iter().cloned().collect())
        ));
    }

    #[test]
    fn matches_true() {
        assert!(Text::matches(
            Text("".into(), Marks::BOLD, vec![].iter().cloned().collect()),
            Text("".into(), Marks::BOLD, vec![].iter().cloned().collect())
        ));
    }

    #[test]
    fn matches_partial_false() {
        assert!(!Text::matches(
            Text(
                "".into(),
                Marks::BOLD | Marks::ITALIC,
                vec![].iter().cloned().collect()
            ),
            Text(
                "".into(),
                Marks::UNDERLINE,
                vec![].iter().cloned().collect()
            )
        ));
    }

    #[test]
    fn matches_partial_true() {
        assert!(Text::matches(
            Text(
                "".into(),
                Marks::BOLD | Marks::ITALIC,
                vec![].iter().cloned().collect()
            ),
            Text("".into(), Marks::BOLD, vec![].iter().cloned().collect())
        ));
    }

    #[test]
    fn decorations_end() {
        let decs = vec![(
            Range::new(
                Point {
                    path: vec![0].into(),
                    offset: 2,
                },
                Point {
                    path: vec![0].into(),
                    offset: 3,
                },
            ),
            vec!["decoration".into()].iter().cloned().collect(),
        )];

        let input = Text(
            "abc".into(),
            Marks::BOLD,
            vec!["test".into()].iter().cloned().collect(),
        );

        assert_eq!(
            input.decorations(decs),
            vec![
                Text(
                    "ab".into(),
                    Marks::BOLD,
                    vec!["test".into()].iter().cloned().collect()
                ),
                Text(
                    "c".into(),
                    Marks::BOLD,
                    vec!["decoration".into(), "test".into()]
                        .iter()
                        .cloned()
                        .collect()
                ),
            ]
        );
    }

    #[test]
    fn decorations_middle() {
        let decs = vec![(
            Range::new(
                Point {
                    path: vec![0].into(),
                    offset: 1,
                },
                Point {
                    path: vec![0].into(),
                    offset: 2,
                },
            ),
            vec!["decoration".into()].iter().cloned().collect(),
        )];

        let input = Text(
            "abc".into(),
            Marks::empty(),
            vec!["test".into()].iter().cloned().collect(),
        );

        assert_eq!(
            input.decorations(decs),
            vec![
                Text(
                    "a".into(),
                    Marks::empty(),
                    vec!["test".into()].iter().cloned().collect()
                ),
                Text(
                    "b".into(),
                    Marks::empty(),
                    vec!["decoration".into(), "test".into()]
                        .iter()
                        .cloned()
                        .collect()
                ),
                Text(
                    "c".into(),
                    Marks::empty(),
                    vec!["test".into()].iter().cloned().collect()
                ),
            ]
        );
    }

    #[test]
    fn decorations_overlapping() {
        let decs = vec![
            (
                Range::new(
                    Point {
                        path: vec![0].into(),
                        offset: 1,
                    },
                    Point {
                        path: vec![0].into(),
                        offset: 2,
                    },
                ),
                vec!["decoration1".into()].iter().cloned().collect(),
            ),
            (
                Range::new(
                    Point {
                        path: vec![0].into(),
                        offset: 0,
                    },
                    Point {
                        path: vec![0].into(),
                        offset: 3,
                    },
                ),
                vec!["decoration2".into()].iter().cloned().collect(),
            ),
        ];

        let input = Text("abc".into(), Marks::BOLD, vec![].iter().cloned().collect());

        assert_eq!(
            input.decorations(decs),
            vec![
                Text(
                    "a".into(),
                    Marks::BOLD,
                    vec!["decoration2".into()].iter().cloned().collect()
                ),
                Text(
                    "b".into(),
                    Marks::BOLD,
                    vec!["decoration1".into(), "decoration2".into()]
                        .iter()
                        .cloned()
                        .collect()
                ),
                Text(
                    "c".into(),
                    Marks::BOLD,
                    vec!["decoration2".into()].iter().cloned().collect()
                ),
            ]
        );
    }

    #[test]
    fn decorations_start() {
        let decs = vec![(
            Range::new(
                Point {
                    path: vec![0].into(),
                    offset: 0,
                },
                Point {
                    path: vec![0].into(),
                    offset: 1,
                },
            ),
            vec!["decoration".into()].iter().cloned().collect(),
        )];

        let input = Text("abc".into(), Marks::BOLD, HashSet::new());

        assert_eq!(
            input.decorations(decs),
            vec![
                Text(
                    "a".into(),
                    Marks::BOLD,
                    vec!["decoration".into()].iter().cloned().collect()
                ),
                Text("bc".into(), Marks::BOLD, HashSet::new()),
            ]
        );
    }
}
