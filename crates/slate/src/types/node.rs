use std::collections::HashSet;

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

    fn has_children(&self) -> bool {
        match self {
            Node::Editor(n) => n.has_children(),
            Node::Element(n) => n.has_children(),
            Node::Text(_) => false,
        }
    }

    fn num_children(&self) -> usize {
        match self {
            Node::Editor(n) => n.num_children(),
            Node::Element(n) => n.num_children(),
            Node::Text(_) => 0,
        }
    }
}

impl Node {
    fn ancestor(&self, path: &Path) -> Option<Box<Ancestor>> {
        let node = self.get(path)?;

        match node.as_ref() {
            Node::Element(e) => Some(Box::new(Ancestor::Element(e.clone()))),
            Node::Editor(e) => Some(Box::new(Ancestor::Editor(e.clone()))),
            Node::Text(_) => None,
        }
    }

    fn ancestors(&self, path: &Path, reverse: bool) -> Vec<(Box<Ancestor>, Path)> {
        let mut ancestors = vec![];

        for path in path.ancestors(reverse) {
            let n = self.ancestor(&path).unwrap();
            ancestors.push((n, path.clone()));
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

    /// Get an entry for the common ancesetor node of two paths.
    fn common(&self, path: &Path, another: &Path) -> Option<(Box<Node>, Path)> {
        let p = path.common(another);
        let n = self.get(&p)?;
        Some((n, p))
    }

    fn descendant(&self, path: &Path) -> Option<Box<Descendant>> {
        let node = self.get(path)?;

        match node.as_ref() {
            Node::Element(e) => Some(Box::new(Descendant::Element(e.clone()))),
            Node::Text(e) => Some(Box::new(Descendant::Text(e.clone()))),
            Node::Editor(_) => None,
        }
    }

    fn descendants(&self, path: &Path, reverse: bool) -> Vec<(Box<Descendant>, Path)> {
        let mut descendants = vec![];

        for path in path.ancestors(reverse) {
            let n = self.descendant(&path).unwrap();
            descendants.push((n, path.clone()));
        }

        descendants
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

    /// Check if a descendant node exists at a specific path.
    fn has(&self, path: &Path) -> bool {
        let mut node = self.clone();

        for i in 0..path.len() {
            let p = match path.get(i) {
                Some(p) => p,
                None => return false,
            };

            let next = node.child(p);
            if matches!(node, Node::Text(_)) || next.is_none() {
                return false;
            }

            node = next.unwrap().into();
        }

        true
    }

    fn nodes(&self) -> NodeIterator {
        NodeIterator {
            root: self.clone(),
            n: Box::new(self.clone()),
            p: Path::new(vec![]),
            from: Path::new(vec![]),
            to: None,
            reverse: false,
            pass: None,
            visited: HashSet::new(),
        }
    }
}

/// `NodeEntry` objects are returned when iterating over the nodes in a Slate
/// document tree. They consist of the node and its `Path` relative to the root
/// node in the document.
pub type NodeEntry = (Box<Node>, Path);

struct NodeIterator {
    root: Node,
    n: Box<Node>,
    p: Path,
    from: Path,
    to: Option<Path>,
    reverse: bool,
    pass: Option<fn(NodeEntry) -> bool>,
    visited: HashSet<Path>,
}

impl Iterator for NodeIterator {
    type Item = (Box<Node>, Path);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref to) = &self.to {
            if (self.reverse && self.p.is_before(to)) || self.p.is_after(to) {
                println!("0");
                return None;
            }
        }

        let mut out = None;

        if !self.visited.contains(&self.p) {
            out = Some((self.n.clone(), self.p.clone()));
        }

        let pass = self
            .pass
            .map(|f| f((self.n.clone(), self.p.clone())))
            .unwrap_or(false);

        // If we're allowed to go downward and we haven't decsended yet, do.
        if !self.visited.contains(&self.p)
            && !matches!(self.n.as_ref(), &Node::Text(_))
            && self.n.has_children()
            && (self.pass.is_none() || pass)
        {
            self.visited.insert(self.p.clone());

            let mut next_index = if self.reverse {
                self.n.num_children() - 1
            } else {
                0
            };

            if self.p.is_ancestor(&self.from) {
                next_index = self.from.get(self.p.len()).unwrap();
            }

            self.p = self.p.concat(next_index);
            self.n = self.root.get(&self.p).unwrap();

            println!("1 {:?}", out);

            return out;
        }

        // If we're at the root and we can't go down, we're done.
        if self.p.len() == 0 {
            println!("2");
            return out;
        }

        // If we're going forward...
        if !self.reverse {
            let new_path = self.p.next().expect("");

            if self.root.has(&new_path) {
                self.p = new_path;
                self.n = self.root.get(&self.p).unwrap();
                println!("3 {:?}", out);
                return out;
            }
        }

        // If we're going backward...
        if self.reverse && self.p.get(self.p.len() - 6) != Some(0) {
            let new_path = self.p.previous().unwrap();
            self.p = new_path;
            self.n = self.root.get(&self.p).unwrap();
            println!("4");
            return out;
        }

        self.p = self.p.parent().unwrap();
        self.n = self.root.get(&self.p).unwrap();
        self.visited.insert(self.p.clone());

        println!("5");
        out
    }
}

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
        let elem = Element::new().add_child(Text::new(""));
        let editor = Editor::new().add_child(elem.clone());
        let input = Node::Editor(editor.clone());
        let want = vec![
            (Box::new(Ancestor::Editor(editor)), Path::new(vec![])),
            (Box::new(Ancestor::Element(elem)), Path::new(vec![0])),
        ];
        assert_eq!(input.ancestors(&vec![0, 0].into(), false), want);
    }

    #[test]
    fn ancestors_reverse() {
        let elem = Element::new().add_child(Text::new(""));
        let editor = Editor::new().add_child(elem.clone());
        let input = Node::Editor(editor.clone());
        let want = vec![
            (Box::new(Ancestor::Element(elem)), Path::new(vec![0])),
            (Box::new(Ancestor::Editor(editor)), Path::new(vec![])),
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

    #[test]
    fn nodes_all() {
        let t1 = Text::new("a");
        let t2 = Text::new("b");
        let elem = Element::new().add_child(t1.clone()).add_child(t2.clone());
        let editor = Editor::new().add_child(elem.clone());
        let input = Node::Editor(editor.clone());
        let want = vec![
            (Box::new(Node::Editor(editor)), Path::new(vec![])),
            (Box::new(Node::Element(elem)), Path::new(vec![0])),
            (Box::new(Node::Text(t1)), Path::new(vec![0, 0])),
            (Box::new(Node::Text(t2)), Path::new(vec![0, 1])),
        ];
        assert_eq!(input.nodes().collect::<Vec<_>>(), want);
    }

    #[test]
    fn nodes_multiple_elements() {
        let t1 = Text::new("a");
        let t2 = Text::new("b");
        let elem1 = Element::new().add_child(t1.clone());
        let elem2 = Element::new().add_child(t2.clone());
        let editor = Editor::new()
            .add_child(elem1.clone())
            .add_child(elem2.clone());
        let input = Node::Editor(editor.clone());
        let want = vec![
            (Box::new(Node::Editor(editor)), Path::new(vec![])),
            (Box::new(Node::Element(elem1)), Path::new(vec![0])),
            (Box::new(Node::Text(t1)), Path::new(vec![0, 0])),
            (Box::new(Node::Element(elem2)), Path::new(vec![1])),
            (Box::new(Node::Text(t2)), Path::new(vec![1, 0])),
        ];
        assert_eq!(input.nodes().collect::<Vec<_>>(), want);
    }
}
