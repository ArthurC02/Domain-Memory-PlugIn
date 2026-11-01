## What changes

## Checklist

- [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass in `engine/`
- [ ] `cargo test --release --locked --test expected_output` passes, or the recording was re-blessed and the diff is explained above
- [ ] A change to what ships raises `version` in `.claude-plugin/plugin.json`, rebuilds `engine/bin/`, and adds a `CHANGELOG.md` entry
