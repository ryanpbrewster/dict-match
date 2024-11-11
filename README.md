# dict-match
Exploration of algorithms for matching dictionaries against rule lists

```
linear_no_match         time:   [7.9907 µs 8.0662 µs 8.1660 µs]
tree_no_match           time:   [126.34 ns 126.71 ns 127.12 ns]
```

Using a tree-structure for low-cardinality rule sets can be substantially faster.