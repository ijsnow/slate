use super::{editor::Editor, element::Element, path::Path, text::Text};

#[derive(Debug, Clone, PartialEq)]
pub enum Descendant {
    Text(Text),
    Element(Element),
}

impl Into<Node> for &Descendant {
    fn into(self) -> Node {
        match self {
            Descendant::Text(t) => Node::Text(t.clone()),
            Descendant::Element(e) => Node::Element(e.clone()),
        }
    }
}

impl Into<Node> for Descendant {
    fn into(self) -> Node {
        match self {
            Descendant::Text(t) => Node::Text(t),
            Descendant::Element(e) => Node::Element(e),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ancestor {
    Editor(Editor),
    Element(Element),
}

impl Ancestor {
    fn children(&self) -> Vec<Descendant> {
        match self {
            Ancestor::Editor(e) => e.children(),
            Ancestor::Element(e) => e.children(),
        }
    }

    fn child(&self, i: usize) -> Option<Descendant> {
        match self {
            Ancestor::Editor(n) => n.child(i).map(|desc| desc.clone()),
            Ancestor::Element(n) => n.child(i).map(|desc| desc.clone()),
        }
    }
}

impl Into<Node> for Box<Ancestor> {
    fn into(self) -> Node {
        match self.as_ref() {
            Ancestor::Editor(e) => Node::Editor(e.clone()),
            Ancestor::Element(e) => Node::Element(e.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Editor(Editor),
    Element(Element),
    Text(Text),
}

impl Node {
    fn child_node(&self, i: usize) -> Option<Node> {
        match self {
            Node::Editor(n) => n.child(i).map(|desc| desc.into()),
            Node::Element(n) => n.child(i).map(|desc| desc.into()),
            Node::Text(_) => None,
        }
    }
}

impl Node {
    /// Get the node at a specific path, asserting that it's an ancestor node.
    fn ancestor(&self, path: &Path) -> Option<Box<Ancestor>> {
        let node = self.get(path)?;

        match node.as_ref() {
            Node::Element(e) => Some(Box::new(Ancestor::Element(e.clone()))),
            Node::Editor(e) => Some(Box::new(Ancestor::Editor(e.clone()))),
            Node::Text(_) => None,
        }
    }

    fn ancestors(&self, path: &Path, reverse: bool) -> Vec<(Node, Path)> {
        let mut ancestors = vec![];

        for path in path.ancestors(reverse) {
            let n = self.ancestor(&path).unwrap();
            ancestors.push((n.into(), path.clone()));
        }

        ancestors
    }

    fn child(&self, i: usize) -> Option<Descendant> {
        match self {
            Node::Editor(n) => n.child(i).map(|desc| desc.clone()),
            Node::Element(n) => n.child(i).map(|desc| desc.clone()),
            Node::Text(_) => None,
        }
    }

    fn children(&self, path: &Path, reverse: bool) -> Vec<(Descendant, Path)> {
        let ancestor = self.ancestor(path).unwrap();
        let children = ancestor.children();
        let mut out = vec![];

        for i in 0..children.len() {
            let child = ancestor.child(i).unwrap();
            let child_path = path.concat(i);
            out.push((child, child_path));
        }

        if reverse {
            out.reverse();
        }

        out
    }

    /// Get the descendant node referred to by a specific path. If the path is an
    /// empty array, it refers to the root node itself.
    fn get(&self, path: &Path) -> Option<Box<Node>> {
        let mut node = self.clone();

        for i in 0..path.len() {
            let p = path.get(i)?;

            if let Some(c) = node.child_node(p).take() {
                node = c.clone();
            } else {
                return None;
            }
        }

        Some(Box::new(node))
    }
}

/// `NodeEntry` objects are returned when iterating over the nodes in a Slate
/// document tree. They consist of the node and its `Path` relative to the root
/// node in the document.
pub type NodeEntry = (Box<Node>, Path);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestor_success() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        let want = Box::new(Ancestor::Element(Element::new().add_child(Text::new(""))));
        assert_eq!(input.ancestor(&vec![0].into()).unwrap(), want);
    }

    #[test]
    fn ancestors_success() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        let want = vec![
            (input.clone(), Path::new(vec![])),
            (input.child(0).unwrap().into(), Path::new(vec![0])),
        ];
        assert_eq!(input.ancestors(&vec![0, 0].into(), false), want);
    }

    #[test]
    fn ancestors_reverse() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        let want = vec![
            (input.child(0).unwrap().into(), Path::new(vec![0])),
            (input.clone(), Path::new(vec![])),
        ];
        assert_eq!(input.ancestors(&vec![0, 0].into(), true), want);
    }

    #[test]
    fn child_success() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        assert_eq!(
            input.child(0).unwrap(),
            Descendant::Element(Element::new().add_child(Text::new("")))
        );
    }

    #[test]
    fn children_success() {
        let input = Node::Editor(
            Editor::new().add_child(
                Element::new()
                    .add_child(Text::with_meta("", ["a".into()].iter().cloned().collect()))
                    .add_child(Text::with_meta("", ["b".into()].iter().cloned().collect())),
            ),
        );
        let want = vec![
            (
                Text::with_meta("", ["a".into()].iter().cloned().collect()).into(),
                Path::new(vec![0, 0]),
            ),
            (
                Text::with_meta("", ["b".into()].iter().cloned().collect()).into(),
                Path::new(vec![0, 1]),
            ),
        ];
        assert_eq!(input.children(&vec![0].into(), false), want);
    }

    #[test]
    fn children_reverse() {
        let input = Node::Editor(
            Editor::new().add_child(
                Element::new()
                    .add_child(Text::with_meta("", ["a".into()].iter().cloned().collect()))
                    .add_child(Text::with_meta("", ["b".into()].iter().cloned().collect())),
            ),
        );
        let want = vec![
            (
                Text::with_meta("", ["b".into()].iter().cloned().collect()).into(),
                Path::new(vec![0, 1]),
            ),
            (
                Text::with_meta("", ["a".into()].iter().cloned().collect()).into(),
                Path::new(vec![0, 0]),
            ),
        ];
        assert_eq!(input.children(&vec![0].into(), true), want);
    }

    #[test]
    fn get_root() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        let want = Box::new(input.clone());
        assert_eq!(input.get(&vec![].into()).unwrap(), want);
    }

    #[test]
    fn get_success() {
        let input = Node::Editor(Editor::new().add_child(Element::new().add_child(Text::new(""))));
        let want = Box::new(Node::Element(Element::new().add_child(Text::new(""))));
        assert_eq!(input.get(&vec![0].into()).unwrap(), want);
    }
}
