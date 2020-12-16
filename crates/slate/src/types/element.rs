use super::node::Descendant;
use super::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    children: Vec<Descendant>,
}

impl Into<Descendant> for Element {
    fn into(self) -> Descendant {
        Descendant::Element(self)
    }
}

impl Element {
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    pub fn add_child(mut self, child: impl Into<Descendant>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn child(&self, i: usize) -> Option<&Descendant> {
        self.children.get(i)
    }

    pub fn children(&self) -> Vec<Descendant> {
        self.children.clone()
    }

    pub fn has_children(&self) -> bool {
        self.children.len() > 0
    }

    pub fn num_children(&self) -> usize {
        self.children.len()
    }
}

/// `ElementEntry` objects refer to an `Element` and the `Path` where it can be
/// found inside a root node.
pub type ElementEntry = (Element, Path);
