use std::cmp::{min, Ord, Ordering};

use super::operation::Operation;

pub enum Affinity {
    Forward,
    Backward,
    None,
}

impl Default for Affinity {
    fn default() -> Self {
        Affinity::Forward
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path(Vec<usize>);

impl Into<Path> for Vec<usize> {
    fn into(self) -> Path {
        Path(self)
    }
}

impl Path {
    pub fn concat(&self, i: usize) -> Self {
        let mut copy = self.clone();
        copy.0.push(i);
        copy
    }
}

impl Path {
    pub fn new(inner: Vec<usize>) -> Self {
        Self(inner)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, i: usize) -> Option<usize> {
        self.0.get(i).map(|p| *p)
    }

    /// Get a list of ancestor paths for a given path.
    pub fn ancestors(&self, reverse: bool) -> Vec<Path> {
        let mut paths = self.levels(reverse);

        if reverse {
            paths = paths[1..].into();
        } else {
            paths = paths[..paths.len() - 1].into();
        }

        paths
    }

    /// Get the common ancestor path of two paths.
    pub fn common(&self, other: &Path) -> Path {
        let mut common = vec![];
        for i in 0..self.0.len() {
            if self.0[i] != other.0[i] {
                break;
            }
            common.push(self.0[i]);
        }
        Path(common)
    }

    fn ends_after(&self, other: &Path) -> bool {
        let i = self.0.len() - 1;
        if i > other.0.len() {
            return false;
        }
        let a_s = &self.0[0..i];
        let b_s = &other.0[0..i];
        let av = self.0[i];
        let bv = other.0[i];
        Path(a_s.into()) == Path(b_s.into()) && av > bv
    }

    fn ends_at(&self, Path(b): &Path) -> bool {
        let i = self.0.len();
        if i > b.len() {
            return false;
        }
        let a_s = &self.0[0..i];
        let b_s = &b[0..i];
        Path(a_s.into()) == Path(b_s.into())
    }

    fn ends_before(&self, Path(b): &Path) -> bool {
        let i = self.0.len() - 1;
        if i >= b.len() {
            return false;
        }
        let a_s = &self.0[0..i];
        let b_s = &b[0..i];
        let av = self.0[i];
        let bv = b[i];
        Path(a_s.into()) == Path(b_s.into()) && av < bv
    }

    fn has_previous(&self) -> bool {
        self.0[self.0.len() - 1] > 0
    }

    pub fn is_after(&self, other: &Path) -> bool {
        self > other
    }

    pub fn is_ancestor(&self, b: &Path) -> bool {
        self.0.len() < b.0.len() && self.cmp(&b) == Ordering::Equal
    }

    pub fn is_before(&self, b: &Path) -> bool {
        self < b
    }

    fn is_child(&self, b: &Path) -> bool {
        self.0.len() == b.0.len() + 1 && self.cmp(&b) == Ordering::Equal
    }

    fn is_common(&self, b: &Path) -> bool {
        self.0.len() <= b.0.len() && self.cmp(&b) == Ordering::Equal
    }

    fn is_descendant(&self, other: &Path) -> bool {
        self.0.len() > other.0.len() && self.cmp(&other) == Ordering::Equal
    }

    fn is_parent(&self, other: &Path) -> bool {
        self.0.len() + 1 == other.0.len() && self.cmp(&other) == Ordering::Equal
    }

    pub fn is_sibling(&self, other: &Path) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let a_s = &self.0[..self.0.len() - 1];
        let b_s = &other.0[..self.0.len() - 1];
        let a_l = self.0[self.0.len() - 1];
        let b_l = other.0[other.0.len() - 1];

        a_l != b_l && Path(a_s.into()) == Path(b_s.into())
    }

    fn levels(&self, reverse: bool) -> Vec<Path> {
        let mut list: Vec<Path> = vec![];
        for i in 0..=self.0.len() {
            list.push(Path(self.0[..i].into()));
        }
        if reverse {
            list.reverse();
        }
        list
    }

    pub fn next(&self) -> Option<Path> {
        if self.0.len() == 0 {
            return None;
        }

        let last = self.0[self.0.len() - 1];
        let mut n: Vec<_> = self.0[..self.0.len() - 1].into();
        n.push(last + 1);

        Some(Path(n))
    }

    pub fn parent(&self) -> Option<Path> {
        if self.0.len() == 0 {
            return None;
        }

        Some(Path(self.0[..self.0.len() - 1].into()))
    }

    pub fn previous(&self) -> Option<Path> {
        if self.0.len() == 0 {
            return None;
        }

        let last = self.0[self.0.len() - 1];
        let prev = last.checked_sub(1)?;
        let mut n: Vec<_> = self.0[..self.0.len() - 1].into();
        n.push(prev);

        Some(Path(n))
    }

    /// Get a path relative to an ancestor.
    fn relative(&self, ancestor: &Path) -> Option<Path> {
        if !Path::is_ancestor(ancestor, self) && self != ancestor {
            return None;
        }

        let (Path(path), Path(ancestor)) = (self, ancestor);

        Some(Path(path[ancestor.len()..].into()))
    }

    pub fn transform(path: &Path, operation: &Operation, affinity: Affinity) -> Option<Path> {
        let mut path = path.clone();

        // PERF: Exit early if the operation is guaranteed not to have an effect.
        if path.0.len() == 0 {
            return Some(path);
        }

        match operation {
            Operation::InsertNode { path: op, .. } => {
                if op == &path || op.ends_before(&path) || op.is_ancestor(&path) {
                    path.0[op.0.len() - 1] += 1;
                }
            }
            Operation::RemoveNode { path: op, .. } => {
                if op == &path || op.is_ancestor(&path) {
                    return None;
                } else if op.ends_before(&path) {
                    path.0[op.0.len() - 1] -= 1
                }
            }
            Operation::MergeNode {
                path: op, position, ..
            } => {
                if op == &path || op.ends_before(&path) {
                    path.0[op.0.len() - 1] -= 1;
                } else if op.is_ancestor(&path) {
                    path.0[op.0.len() - 1] -= 1;
                    path.0[op.0.len()] += position;
                }
            }
            Operation::SplitNode {
                path: op, position, ..
            } => {
                if op == &path {
                    if matches!(affinity, Affinity::Forward) {
                        let i = path.0.len() - 1;
                        path.0[i] += 1;
                    } else if matches!(affinity, Affinity::Backward) {
                        // Nothing, because it still refers to the right path.
                    } else {
                        return None;
                    }
                } else if op.ends_before(&path) {
                    path.0[op.0.len() - 1] += 1;
                } else if op.is_ancestor(&path) && &path.0[op.0.len()] >= position {
                    path.0[op.0.len() - 1] += 1;
                    path.0[op.0.len()] -= position;
                }
            }
            Operation::MoveNode {
                path: op,
                new_path: onp,
            } => {
                let mut onp = onp.clone();
                // If the old and new path are the same, it's a no-op.
                if op == &onp {
                    return Some(path);
                }

                if op.is_ancestor(&path) || op == &path {
                    if op.ends_before(&onp) && op.0.len() < onp.0.len() {
                        onp.0[op.0.len() - 1] -= 1;
                    }

                    let a = &path.0[op.0.len()..];

                    onp.0.extend(a);

                    return Some(onp);
                } else if op.is_sibling(&onp) && (onp.is_ancestor(&path) || onp == path) {
                    if op.ends_before(&path) {
                        path.0[op.0.len() - 1] -= 1;
                    } else {
                        path.0[op.0.len() - 1] += 1;
                    }
                } else if onp.ends_before(&path) || onp == path || onp.is_ancestor(&path) {
                    if op.ends_before(&path) {
                        path.0[op.0.len() - 1] -= 1;
                    }

                    path.0[onp.0.len() - 1] += 1;
                } else if op.ends_before(&path) {
                    if onp == path {
                        path.0[onp.0.len() - 1] += 1;
                    }

                    path.0[op.0.len() - 1] -= 1;
                }
            }
            _ => {}
        }

        Some(path)
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        let len = min(self.0.len(), other.0.len());
        for i in 0..len {
            let cmp = self.0[i].cmp(&other.0[i]);
            if cmp != Ordering::Equal {
                return cmp;
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestors_success() {
        let input = Path(vec![0, 1, 2]);
        assert_eq!(
            Path::ancestors(&input, false),
            vec![Path(vec![]), Path(vec![0]), Path(vec![0, 1])]
        );
    }

    #[test]
    fn ancestors_reverse() {
        let input = Path(vec![0, 1, 2]);
        assert_eq!(
            Path::ancestors(&input, true),
            vec![Path(vec![0, 1]), Path(vec![0]), Path(vec![])]
        );
    }

    #[test]
    fn common_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::common(&a, &b), Path(vec![0, 1, 2]));
    }

    #[test]
    fn common_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![3, 2]);
        assert_eq!(Path::common(&a, &b), Path(vec![]));
    }

    #[test]
    fn common_success() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::common(&a, &b), Path(vec![0]));
    }

    #[test]
    fn compare_above() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn compare_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert!(a > b);
    }

    #[test]
    fn compare_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert!(a < b);
    }

    #[test]
    fn compare_below() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn compare_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn compare_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![]);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ends_after_above() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_below() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_ends_after() {
        let a = Path(vec![1]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_after(&a, &b), true);
    }

    #[test]
    fn ends_after_ends_at() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_ends_before() {
        let a = Path(vec![0]);
        let b = Path(vec![1, 2]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_after_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![]);
        assert_eq!(Path::ends_after(&a, &b), false);
    }

    #[test]
    fn ends_at_above() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_at_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_at_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_at_ends_after() {
        let a = Path(vec![1]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_at_ends_at() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_at(&a, &b), true);
    }

    #[test]
    fn ends_at_ends_before() {
        let a = Path(vec![0]);
        let b = Path(vec![1, 2]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_at_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::ends_at(&a, &b), true);
    }

    #[test]
    fn ends_at_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![]);
        assert_eq!(Path::ends_at(&a, &b), false);
    }

    #[test]
    fn ends_before_above() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_below() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_ends_after() {
        let a = Path(vec![1]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_ends_at() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 2]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_ends_before() {
        let a = Path(vec![0]);
        let b = Path(vec![1, 2]);
        assert_eq!(Path::ends_before(&a, &b), true);
    }

    #[test]
    fn ends_before_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn ends_before_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![]);
        assert_eq!(Path::ends_before(&a, &b), false);
    }

    #[test]
    fn equals_above() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert!(a != b);
    }

    #[test]
    fn equals_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert!(a != b);
    }

    #[test]
    fn equals_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert!(a != b);
    }

    #[test]
    fn equals_below() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert!(a != b);
    }

    #[test]
    fn equals_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert!(a == b);
    }

    #[test]
    fn equals_root() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![]);
        assert!(a != b);
    }

    #[test]
    fn has_previous_root() {
        let a = Path(vec![0, 0]);
        assert_eq!(Path::has_previous(&a), false);
    }

    #[test]
    fn has_previous_success() {
        let a = Path(vec![0, 1]);
        assert_eq!(Path::has_previous(&a), true);
    }

    #[test]
    fn is_after_above() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(a.is_after(&b), false);
    }

    #[test]
    fn is_after_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(a.is_after(&b), true);
    }

    #[test]
    fn is_after_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(a.is_after(&b), false);
    }

    #[test]
    fn is_after_below() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(a.is_after(&b), false);
    }

    #[test]
    fn is_after_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(a.is_after(&b), false);
    }

    #[test]
    fn is_ancestor_above_grandparent() {
        let a = Path(vec![]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_ancestor(&a, &b), true);
    }

    #[test]
    fn is_ancestor_above_parent() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_ancestor(&a, &b), true);
    }

    #[test]
    fn is_ancestor_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_ancestor(&a, &b), false);
    }

    #[test]
    fn is_ancestor_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_ancestor(&a, &b), false);
    }

    #[test]
    fn is_ancestor_below() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_ancestor(&a, &b), false);
    }

    #[test]
    fn is_ancestor_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::is_ancestor(&a, &b), false);
    }

    #[test]
    fn is_before_above() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_before(&a, &b), false);
    }

    #[test]
    fn is_before_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_before(&a, &b), false);
    }

    #[test]
    fn is_before_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_before(&a, &b), true);
    }

    #[test]
    fn is_before_below() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_before(&a, &b), false);
    }

    #[test]
    fn is_before_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::is_before(&a, &b), false);
    }

    #[test]
    fn is_child_above() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_child(&a, &b), false);
    }

    #[test]
    fn is_child_after() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_child(&a, &b), false);
    }

    #[test]
    fn is_child_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_child(&a, &b), false);
    }

    #[test]
    fn is_child_below_child() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_child(&a, &b), true);
    }

    #[test]
    fn is_child_below_grandchild() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![]);
        assert_eq!(Path::is_child(&a, &b), false);
    }

    #[test]
    fn is_child_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::is_child(&a, &b), false);
    }

    #[test]
    fn is_descendant_above() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_descendant(&a, &b), false);
    }

    #[test]
    fn is_descendant_after() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_descendant(&a, &b), false);
    }

    #[test]
    fn is_descendant_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_descendant(&a, &b), false);
    }

    #[test]
    fn is_descendant_below_child() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_descendant(&a, &b), true);
    }

    #[test]
    fn is_descendant_below_grandchild() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![]);
        assert_eq!(Path::is_descendant(&a, &b), true);
    }

    #[test]
    fn is_descendant_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::is_descendant(&a, &b), false);
    }

    #[test]
    fn is_parent_above_grandparent() {
        let a = Path(vec![]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_parent(&a, &b), false);
    }

    #[test]
    fn is_parent_above_parent() {
        let a = Path(vec![0]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_parent(&a, &b), true);
    }

    #[test]
    fn is_parent_after() {
        let a = Path(vec![1, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_parent(&a, &b), false);
    }

    #[test]
    fn is_parent_before() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_parent(&a, &b), false);
    }

    #[test]
    fn is_parent_below() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_parent(&a, &b), false);
    }

    #[test]
    fn is_parent_equal() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0, 1, 2]);
        assert_eq!(Path::is_parent(&a, &b), false);
    }

    #[test]
    fn is_sibling_above() {
        let a = Path(vec![]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_sibling(&a, &b), false);
    }

    #[test]
    fn is_sibling_after_sibling() {
        let a = Path(vec![1, 4]);
        let b = Path(vec![1, 2]);
        assert_eq!(Path::is_sibling(&a, &b), true);
    }

    #[test]
    fn is_sibling_after() {
        let a = Path(vec![1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_sibling(&a, &b), false);
    }

    #[test]
    fn is_sibling_before_sibling() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![0, 3]);
        assert_eq!(Path::is_sibling(&a, &b), true);
    }

    #[test]
    fn is_sibling_before() {
        let a = Path(vec![0, 2]);
        let b = Path(vec![1]);
        assert_eq!(Path::is_sibling(&a, &b), false);
    }

    #[test]
    fn is_sibling_below() {
        let a = Path(vec![0, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::is_sibling(&a, &b), false);
    }

    #[test]
    fn is_sibling_equal() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![0, 1]);
        assert_eq!(Path::is_sibling(&a, &b), false);
    }

    #[test]
    fn levels_success() {
        let input = Path(vec![0, 1, 2]);
        assert_eq!(
            Path::levels(&input, false),
            vec![
                Path(vec![]),
                Path(vec![0]),
                Path(vec![0, 1]),
                Path(vec![0, 1, 2]),
            ]
        );
    }

    #[test]
    fn levels_reverse() {
        let input = Path(vec![0, 1, 2]);
        assert_eq!(
            Path::levels(&input, true),
            vec![
                Path(vec![0, 1, 2]),
                Path(vec![0, 1]),
                Path(vec![0]),
                Path(vec![]),
            ]
        );
    }

    #[test]
    fn next_success() {
        let a = Path(vec![0, 1]);
        assert_eq!(Path::next(&a).unwrap(), Path(vec![0, 2]));
    }

    #[test]
    fn parent_success() {
        let a = Path(vec![0, 1]);
        assert_eq!(Path::parent(&a).unwrap(), Path(vec![0]));
    }

    #[test]
    fn previous_success() {
        let a = Path(vec![0, 1]);
        assert_eq!(Path::previous(&a).unwrap(), Path(vec![0, 0]));
    }

    #[test]
    fn relative_grandparent() {
        let a = Path(vec![0, 1, 2]);
        let b = Path(vec![0]);
        assert_eq!(Path::relative(&a, &b).unwrap(), Path(vec![1, 2]));
    }

    #[test]
    fn relative_parent() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![0]);
        assert_eq!(Path::relative(&a, &b).unwrap(), Path(vec![1]));
    }

    #[test]
    fn relative_root() {
        let a = Path(vec![0, 1]);
        let b = Path(vec![]);
        assert_eq!(Path::relative(&a, &b).unwrap(), Path(vec![0, 1]));
    }

    #[test]
    fn transform_ancestor_sibling_ends_after_to_ancestor() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![4]),
            new_path: Path(vec![3]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![4, 3, 3])
        );
    }

    #[test]
    fn transform_ancestor_sibling_ends_after_to_ends_after() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![4]),
            new_path: Path(vec![2]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![4, 3, 3])
        );
    }

    #[test]
    fn transform_ancestor_sibling_ends_before_to_ancestor() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![2]),
            new_path: Path(vec![3]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![2, 3, 3])
        );
    }

    #[test]
    fn transform_ancestor_sibling_ends_before_to_ends_after() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![2]),
            new_path: Path(vec![4]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![2, 3, 3])
        );
    }

    #[test]
    fn transform_ancestor_to_ends_after() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3]),
            new_path: Path(vec![5, 1]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![4, 1, 3, 3])
        );
    }

    #[test]
    fn transform_ancestor_to_ends_before() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3]),
            new_path: Path(vec![2, 5]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![2, 5, 3, 3])
        );
    }

    #[test]
    fn transform_ends_after_to_no_relation() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 4]),
            new_path: Path(vec![3, 0, 0]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 3, 3])
        );
    }

    #[test]
    fn transform_ends_before_to_no_relation() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 2]),
            new_path: Path(vec![3, 0, 0]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 2, 3])
        );
    }

    #[test]
    fn transform_equal_to_ends_after() {
        let path = Path(vec![3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 3]),
            new_path: Path(vec![3, 5, 0]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 4, 0])
        );
    }

    #[test]
    fn transform_equal_to_ends_before() {
        let path = Path(vec![3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 3]),
            new_path: Path(vec![3, 1, 0]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 1, 0])
        );
    }

    #[test]
    fn transform_no_relation_to_ends_after() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 0, 0]),
            new_path: Path(vec![3, 4]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 3, 3])
        );
    }

    #[test]
    fn transform_no_relation_to_ends_before() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 0, 0]),
            new_path: Path(vec![3, 2]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![3, 4, 3])
        );
    }

    #[test]
    fn transform_parent_to_ends_after() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 3]),
            new_path: Path(vec![5, 1]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![5, 1, 3])
        );
    }

    #[test]
    fn transform_parent_to_ends_before() {
        let path = Path(vec![3, 3, 3]);
        let op = Operation::MoveNode {
            path: Path(vec![3, 3]),
            new_path: Path(vec![2, 1]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![2, 1, 3])
        );
    }

    #[test]
    fn transform_sibling_ends_after_to_ends_equal() {
        let path = Path(vec![0, 1]);
        let op = Operation::MoveNode {
            path: Path(vec![0, 3]),
            new_path: Path(vec![0, 1]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![0, 2])
        );
    }

    #[test]
    fn transform_sibling_ends_after_to_sibling_ends_before() {
        let path = Path(vec![0, 1]);
        let op = Operation::MoveNode {
            path: Path(vec![0, 3]),
            new_path: Path(vec![0, 0]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![0, 2])
        );
    }

    #[test]
    fn transform_sibling_ends_before_to_ends_equal() {
        let path = Path(vec![0, 1]);
        let op = Operation::MoveNode {
            path: Path(vec![0, 0]),
            new_path: Path(vec![0, 1]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![0, 0])
        );
    }

    #[test]
    fn transform_sibling_ends_before_to_sibling_ends_after() {
        let path = Path(vec![0, 1]);
        let op = Operation::MoveNode {
            path: Path(vec![0, 0]),
            new_path: Path(vec![0, 3]),
        };
        assert_eq!(
            Path::transform(&path, &op, Default::default()).unwrap(),
            Path(vec![0, 0])
        );
    }
}
