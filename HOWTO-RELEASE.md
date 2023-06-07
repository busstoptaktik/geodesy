# DRAFT: How to release

- update `Cargo.toml` with new version id, e.g. `"0.11.0"`
- `just check-all`
- `just changes` generates a new `CHANGELOG`
- `git commit ...`
- `git push`
- `git tag v0.11.0`
- `git push --tags`
- `git branch 0.11`
- `git switch 0.11`
- `git push ...`
- `git switch main`
- `cargo publish`
