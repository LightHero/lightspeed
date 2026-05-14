
# Run all tests
[group('test')]
test test_name="":
  cargo test {{test_name}}
  cargo test {{test_name}} --all-features
