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
    fn find(&self, input: &BTreeMap<String, String>) -> Option<usize>;
}

/// LinearScan will walk through the rules in order and return the index of the first
/// rule that matches. This is extremely simple to implement and very fast when there are
/// not many rules.
///
/// The worst-case behavior here is when the input doesn't match any of the rules
/// because it has to check every rule.
pub struct LinearScan(Vec<Rule>);
impl Matcher for LinearScan {
    fn new(rules: Vec<Rule>) -> Self {
        Self(rules)
    }

    fn find(&self, input: &BTreeMap<String, String>) -> Option<usize> {
        for (i, rule) in self.0.iter().enumerate() {
            if rule.0.iter().all(|(k, v)| input.get(k) == Some(v)) {
                return Some(i);
            }
        }
        None
    }
}

/// LowCardinalityTree is an optimization for large rule lists where the rules have
/// a small number of distinct keys but many distinct values (e.g., rule_1 -> priority=p1,
/// rule_2 -> priority=p2, ..., rule_n -> priority=pn). Rather than doing a linear scan,
/// we can index all of the "relevant" values for each key.
///
/// The overall runtime here is Ω(2^K) where K is the number of distinct keys
/// across all rules in the rule list. This structure only makes sense to use when K << N.
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
    },
}
impl Node {
    fn empty() -> Box<Node> {
        Box::new(Node::Parent {
            wildcard: None,
            explicit: BTreeMap::new(),
        })
    }
    fn find_min_leaf(&self, keys: &[String], input: &BTreeMap<String, String>) -> Option<usize> {
        match self {
            Node::Leaf(idx) => Some(*idx),
            Node::Parent { wildcard, explicit } => {
                let mut best = None;
                if let Some(child) = wildcard {
                    best = min_some(best, child.find_min_leaf(&keys[1..], input));
                }
                if let Some(value) = input.get(&keys[0]) {
                    if let Some(child) = explicit.get(value) {
                        best = min_some(best, child.find_min_leaf(&keys[1..], input));
                    }
                }
                best
            }
        }
    }
}
impl Matcher for LowCardinalityTree {
    fn new(rules: Vec<Rule>) -> Self {
        let keys: Vec<String> = {
            let deduped = rules
                .iter()
                .flat_map(|r| r.0.keys().cloned())
                .collect::<BTreeSet<_>>();
            deduped.into_iter().collect()
        };
        if keys.is_empty() {
            return Self {
                keys,
                root: if rules.is_empty() {
                    None
                } else {
                    Some(Node::Leaf(0))
                },
            };
        }

        let mut root = Node::Parent {
            wildcard: None,
            explicit: BTreeMap::new(),
        };
        for (rule_idx, rule) in rules.into_iter().enumerate() {
            let mut cur = &mut root;
            for key in keys.iter() {
                let Node::Parent { wildcard, explicit } = cur else {
                    unreachable!("no interior leaves")
                };
                cur = match rule.0.get(key) {
                    None => wildcard.get_or_insert_with(Node::empty),
                    Some(value) => explicit.entry(value.to_owned()).or_insert_with(Node::empty),
                }
            }
            *cur = Node::Leaf(rule_idx);
        }
        Self {
            keys,
            root: Some(root),
        }
    }

    fn find(&self, input: &BTreeMap<String, String>) -> Option<usize> {
        self.root.as_ref()?.find_min_leaf(&self.keys, input)
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
            Rule(bag! { "a" => "1", "b" => "2" }),
            Rule(bag! { "a" => "1" }),
            Rule(bag! { "b" => "2" }),
        ]);

        assert_eq!(
            m.find(&bag! { "a" => "1", "b" => "2", "c" => "3" }),
            Some(0)
        );
        assert_eq!(
            m.find(&bag! { "a" => "1", "b" => "garbage", "c" => "3" }),
            Some(1)
        );
        assert_eq!(
            m.find(&bag! { "a" => "garbage", "b" => "2", "c" => "3" }),
            Some(2)
        );
        assert_eq!(
            m.find(&bag! { "a" => "garbage", "b" => "garbage", "c" => "3" }),
            None
        );
    }

    #[test]
    fn low_cardinality_tree_smoke() {
        let m = LowCardinalityTree::new(vec![
            Rule(bag! { "a" => "1", "b" => "2" }),
            Rule(bag! { "a" => "1" }),
            Rule(bag! { "b" => "2" }),
        ]);

        assert_eq!(
            m.find(&bag! { "a" => "1", "b" => "2", "c" => "3" }),
            Some(0)
        );
        assert_eq!(
            m.find(&bag! { "a" => "1", "b" => "garbage", "c" => "3" }),
            Some(1)
        );
        assert_eq!(
            m.find(&bag! { "a" => "garbage", "b" => "2", "c" => "3" }),
            Some(2)
        );
        assert_eq!(
            m.find(&bag! { "a" => "garbage", "b" => "garbage", "c" => "3" }),
            None
        );
    }
}
