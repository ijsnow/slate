use super::node::Node;
use super::path::Path;
use super::range::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    InsertNode {
        node: Node,
        path: Path,
    },
    InsertText {
        path: Path,
        offset: usize,
        text: String,
    },
    MergeNode {
        path: Path,
        position: usize,
        properties: Node,
    },
    MoveNode {
        path: Path,
        new_path: Path,
    },
    RemoveNode {
        path: Path,
        node: Node,
    },
    RemoveText {
        path: Path,
        offset: usize,
        text: String,
    },
    SetNode {
        path: Path,
        properties: Option<Node>,
        new_properties: Option<Node>,
    },
    SetSelection {
        path: Path,
        properties: Option<Range>,
        new_properties: Option<Range>,
    },
    SplitNode {
        path: Path,
        position: usize,
        properties: Node,
    },
}

impl Operation {
    pub fn inverse(self) -> Self {
        match self {
            Operation::InsertNode { node, path } => Operation::RemoveNode { node, path },
            Operation::InsertText { path, offset, text } => {
                Operation::RemoveText { path, offset, text }
            }
            Operation::MergeNode {
                path,
                position,
                properties,
            } => Operation::SplitNode {
                path: path.previous().expect("to have previous"),
                position,
                properties,
            },
            Operation::MoveNode { path, new_path } => {
                // PERF: in this case the move operation is a no-op anyways.
                if path == new_path {
                    return Operation::MoveNode { path, new_path };
                }

                // If the move happens completely within a single parent the path and
                // newPath are stable with respect to each other.
                if path.is_sibling(&new_path) {
                    return Operation::MoveNode {
                        path: new_path,
                        new_path: path,
                    };
                }

                // If the move does not happen within a single parent it is possible
                // for the move to impact the true path to the location where the node
                // was removed from and where it was inserted. We have to adjust for this
                // and find the original path. We can accomplish this (only in non-sibling)
                // moves by looking at the impact of the move operation on the node
                // after the original move path.
                let inverse_path = Path::transform(
                    &path,
                    &Operation::MoveNode {
                        path: path.clone(),
                        new_path: new_path.clone(),
                    },
                    Default::default(),
                )
                .unwrap();
                let inverse_new_path = Path::transform(
                    &path.next().unwrap(),
                    &Operation::MoveNode { path, new_path },
                    Default::default(),
                )
                .unwrap();

                Operation::MoveNode {
                    path: inverse_path,
                    new_path: inverse_new_path,
                }
            }
            Operation::RemoveNode { node, path } => Operation::InsertNode { node, path },
            Operation::RemoveText { path, offset, text } => {
                Operation::InsertText { path, offset, text }
            }
            Operation::SetNode {
                path,
                properties,
                new_properties,
            } => Operation::SetNode {
                path,
                properties: new_properties,
                new_properties: properties,
            },
            Operation::SetSelection {
                path,
                properties,
                new_properties,
            } => {
                if properties.is_none() {
                    Operation::SetSelection {
                        path,
                        properties: new_properties,
                        new_properties: None,
                    }
                } else if new_properties.is_none() {
                    Operation::SetSelection {
                        path,
                        properties: None,
                        new_properties: properties,
                    }
                } else {
                    Operation::SetSelection {
                        path,
                        properties: new_properties,
                        new_properties: properties,
                    }
                }
            }
            Operation::SplitNode {
                path,
                position,
                properties,
            } => Operation::MergeNode {
                path: path.next().expect("to have next"),
                position,
                properties,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::path::Path;
    use super::*;

    #[test]
    fn inverse_move_node_backward_in_parent() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 2]),
            new_path: Path::new(vec![0, 1]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 1]),
                new_path: Path::new(vec![0, 2]),
            }
        );
    }

    #[test]
    fn inverse_move_node_child_to_ends_after_parent() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 2, 1]),
            new_path: Path::new(vec![0, 3]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 3]),
                new_path: Path::new(vec![0, 2, 1]),
            }
        );
    }

    #[test]
    fn inverse_move_node_child_to_ends_before_parent() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 2, 1]),
            new_path: Path::new(vec![0, 1]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 1]),
                new_path: Path::new(vec![0, 3, 1]),
            }
        );
    }

    #[test]
    fn inverse_move_node_child_to_parent() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 2, 1]),
            new_path: Path::new(vec![0, 2]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 2]),
                new_path: Path::new(vec![0, 3, 1]),
            }
        );
    }

    #[test]
    fn inverse_move_node_ends_after_parent_to_child() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 3]),
            new_path: Path::new(vec![0, 2, 1]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 2, 1]),
                new_path: Path::new(vec![0, 3]),
            }
        );
    }

    #[test]
    fn inverse_move_node_ends_before_parent_to_child() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 1]),
            new_path: Path::new(vec![0, 2, 1]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 1, 1]),
                new_path: Path::new(vec![0, 1]),
            }
        );
    }

    #[test]
    fn inverse_move_node_forward_in_parent() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 1]),
            new_path: Path::new(vec![0, 2]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![0, 2]),
                new_path: Path::new(vec![0, 1]),
            }
        );
    }

    #[test]
    fn inverse_move_node_non_sibling() {
        let op = Operation::MoveNode {
            path: Path::new(vec![0, 2]),
            new_path: Path::new(vec![1, 0, 0]),
        };
        assert_eq!(
            op.inverse(),
            Operation::MoveNode {
                path: Path::new(vec![1, 0, 0]),
                new_path: Path::new(vec![0, 2]),
            }
        );
    }
}
