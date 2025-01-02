
# Run all tests
[group('test')]
test test_name="":
  cargo test {{test_name}}
  just for_each_feature "cargo test {{test_name}} --features "
  cargo test {{test_name}} --all-features
