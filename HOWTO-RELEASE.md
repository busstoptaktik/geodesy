# DRAFT: How to release

- `git switch main`
- `just clean-check`

- manually check that all [issues](https://github.com/busstoptaktik/geodesy/issues/)
  assigned to the
  [milestone for the upcomming release](https://github.com/busstoptaktik/geodesy/issues?q=is%3Aopen+is%3Aissue+milestone%3A0.14.0)
  are resolved
- update `Cargo.toml` with new version id, i.e. `"0.14.0"`

- `just changes` (to preview a new `CHANGELOG`)
- manually update `CHANGELOG.md` (mostly: change unreleased to 0.14.0)
- `git commit -a -m "CHANGELOG.md for v0.14.0"`
- `git push`
- `git tag v0.14.0`
- `git push --tags`
- `git branch 0.14`
- `git switch 0.14`
- `git push --set-upstream origin 0.14`
- `git switch main`
- `cargo publish`
- update `HOWTO-RELEASE.md` to say 0.15
- `git commit -a -m "Start of work towards 0.15.0"`
- `git push ...`

Twitter/Mastodon/DiscordGeo:

```txt
Rust Geodesy version 0.14.0 just released. X months and Y commits in the making. Get it while it's hot! https://lib.rs/geodesy | https://crates.io/crates/geodesy | https://docs.rs/geodesy/latest/geodesy/
```

Also post new section of Changelog to DiscordGeo
