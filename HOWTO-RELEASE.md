# How to release

- `git switch main`
- `just clean-check`

- manually check that all [issues](https://github.com/busstoptaktik/geodesy/issues/)
  assigned to the
  [milestone for the upcomming release](https://github.com/busstoptaktik/geodesy/issues?q=is%3Aopen+is%3Aissue+milestone%3A0.16.0)
  are resolved
- update `Cargo.toml` with new version id, i.e. `"0.16.0"`

- `just changes` (for new `CHANGELOG.md` material)
- Manually update `CHANGELOG.md`
- `git commit -a -m "CHANGELOG.md for v0.16.0"`
- `git push`
- `git tag v0.16.0`
- `git push --tags`
- `git branch 0.16`
- `git switch 0.16`
- `git push --set-upstream origin 0.16`
- `git switch main`
- `cargo publish`
- Count number of months since last release, X, and number of commits since then, Y
- Announce on Bluesky/Mastodon/DiscordGeo:

   ```txt
   Rust Geodesy version 0.16.0 just released. X months and Y commits in the making. Get it while it's hot!

   https://crates.io/crates/geodesy | https://docs.rs/geodesy/latest/geodesy/

   ```

- `git commit -a -m "Start of work towards 0.17.0"`
- Manually change `HOWTO-RELEASE.md` from 0.17 to 0.18, and from 0.16 to 0.17
- Manually update `CHANGELOG.md` with a new `[Unreleased]` section
- `git commit -a -m "Update HOWTO-RELEASE.md and CHANGELOG.md"`
- `git push`
