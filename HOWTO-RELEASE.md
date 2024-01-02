# DRAFT: How to release

- manually check that all [issues](https://github.com/busstoptaktik/geodesy/issues/)
  assigned to the
  [milestone for upcomming release](https://github.com/busstoptaktik/geodesy/issues?q=is%3Aopen+is%3Aissue+milestone%3A0.12.0)
  are resolved
- update `Cargo.toml` and `README.md` with new version id, i.e. `"0.13.0"`

- `just check-all`
- `just changes` (to preview a new `CHANGELOG`)
- update `CHANGES.md`
- `just changelog` (to generate a new `CHANGELOG`)
- `git commit -a -m "CHANGES.md and CHANGELOG for v0.13.0"`
- `git push`
- `git tag v0.13.0`
- `git push --tags`
- `git branch 0.13`
- `git switch 0.13`
- `git push --set-upstream origin 0.13`
- `git switch main`
- `cargo publish`
- update `HOWTO-RELEASE.md` to say 0.14
- `git commit -a -m "Start of work towards 0.14.0`
- `git push ...`
