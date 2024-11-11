use std::collections::{BTreeMap, BTreeSet};

pub struct Rule(pub BTreeMap<String, String>);

#[macro_export]
macro_rules! bag {
    ($($k:expr => $v:expr),* $(,)?) => {{
        BTreeMap::from([
            $(($k.to_owned(), $v.to_owned()),)*
        ])
    }};
}



pub trait Matcher {
    fn new(rules: Vec<Rule>) -> Self;
    /// find identifies the first rule that matches the input
    fn find(&self, input: BTreeMap<String, String>) -> Option<usize>;
}

pub struct LinearScan(Vec<Rule>);
impl Matcher for LinearScan {
    fn new(rules: Vec<Rule>) -> Self {
        Self(rules)
    }

    fn find(&self, input: BTreeMap<String, String>) -> Option<usize> {
        for (i, rule) in self.0.iter().enumerate() {
            if rule.0.iter().all(|(k, v)| input.get(k) == Some(v)) {
                return Some(i);
            }
        }
        None
    }
}

pub struct LowCardinalityTree {
    keys: Vec<String>,
    root: Option<Node>,
}
#[derive(Debug)]
enum Node {
    Leaf(usize),
    Parent {
        wildcard: Option<Box<Node>>,
        explicit: BTreeMap<String, Box<Node>>,
    }
}
impl Matcher for LowCardinalityTree {
    fn new(rules: Vec<Rule>) -> Self {
        let keys: Vec<String> = {
            let deduped = rules.iter().flat_map(|r| r.0.keys().cloned()).collect::<BTreeSet<_>>();
            deduped.into_iter().collect()
        };
        if keys.is_empty() {
            return Self {
                keys,
                root: if rules.is_empty() { None } else { Some(Node::Leaf(0)) },
            };
        }

        let mut root = Node::Parent { wildcard: None, explicit: BTreeMap::new() };
        for (rule_idx, rule) in rules.into_iter().enumerate() {
            let mut cur = &mut root;
            for (key_idx, key) in keys.iter().enumerate() {
                let child = if key_idx == keys.len() - 1 {
                    Node::Leaf(rule_idx)
                } else {
                    Node::Parent { wildcard: None, explicit: BTreeMap::new() }
                };
                match cur {
                    Node::Leaf(_) => unreachable!("no interior leaves"),
                    Node::Parent { wildcard, explicit } => {
                        match rule.0.get(key) {
                            None => {
                                cur = wildcard.get_or_insert_with(|| Box::new(child));
                            }
                            Some(value) => {
                                cur = explicit.entry(value.to_owned()).or_insert(Box::new(child));
                            }
                        }
                    }
                }
            }
        }
        Self { keys, root: Some(root) }
    }

    fn find(&self, input: BTreeMap<String, String>) -> Option<usize> {
        find_min_leaf(&self.keys, self.root.as_ref()?, &input)
    }
}

fn find_min_leaf(keys: &[String], node: &Node, input: &BTreeMap<String, String>) -> Option<usize> {
    match node {
        Node::Leaf(idx) => Some(*idx),
        Node::Parent { wildcard, explicit } => {
            let mut best = None;
            if let Some(child) = wildcard {
                best = min_some(best, find_min_leaf(&keys[1..], &child, input));
            }
            if let Some(value) = input.get(&keys[0]) {
                if let Some(child) = explicit.get(value) {
                    best = min_some( best, find_min_leaf(&keys[1..], child, input));
                }
            }
            best
        }
    }
}

fn min_some(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_scan_smoke() {
        let m = LinearScan::new(vec![
            Rule(bag!{ "a" => "1", "b" => "2" }),
            Rule(bag!{ "a" => "1" }),
            Rule(bag!{ "b" => "2" }),
        ]);

        assert_eq!(m.find(bag! { "a" => "1", "b" => "2", "c" => "3" }), Some(0));
        assert_eq!(m.find(bag! { "a" => "1", "b" => "garbage", "c" => "3" }), Some(1));
        assert_eq!(m.find(bag! { "a" => "garbage", "b" => "2", "c" => "3" }), Some(2));
        assert_eq!(m.find(bag! { "a" => "garbage", "b" => "garbage", "c" => "3" }), None);
    }

    #[test]
    fn low_cardinality_tree_smoke() {
        let m = LowCardinalityTree::new(vec![
            Rule(bag!{ "a" => "1", "b" => "2" }),
            Rule(bag!{ "a" => "1" }),
            Rule(bag!{ "b" => "2" }),
        ]);

        assert_eq!(m.find(bag! { "a" => "1", "b" => "2", "c" => "3" }), Some(0));
        assert_eq!(m.find(bag! { "a" => "1", "b" => "garbage", "c" => "3" }), Some(1));
        assert_eq!(m.find(bag! { "a" => "garbage", "b" => "2", "c" => "3" }), Some(2));
        assert_eq!(m.find(bag! { "a" => "garbage", "b" => "garbage", "c" => "3" }), None);
    }
}
