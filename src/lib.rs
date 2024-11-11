use std::collections::BTreeMap;

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
}
