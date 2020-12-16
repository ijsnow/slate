use super::node::Descendant;
use super::operation::Operation;
use super::range::Range;
use super::text::Marks;

type Selection = Option<Range>;

#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    children: Vec<Descendant>,
    selection: Selection,
    operations: Vec<Operation>,
    marks: Option<Marks>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            children: vec![],
            selection: None,
            operations: vec![],
            marks: None,
        }
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
