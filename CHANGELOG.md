# Changelog

## 0.2.0

This release contains breaking API changes relative to `0.1.4`, hence the
minor version bump.

### Breaking changes

- `as_slice()` was renamed to `as_str()` on `Style`, `Id`, `Fill` and `Side`.
  Replace `x.as_slice()` with `x.as_str()`.
- `Arrow::default()` is no longer an inherent method; `Arrow` now derives
  `Default`. Existing `Arrow::default()` call sites keep working via the
  `Default` trait, but the method can no longer be referenced as an inherent
  function (e.g. in a path expression expecting an inherent item).
- The crate now uses edition 2024, which requires a recent Rust toolchain.

### Added

- Subgraph support ([#8], [#16]): `Labeller::subgraph_id`, `subgraph_label`,
  `subgraph_style`, `subgraph_shape`, `subgraph_color` and
  `GraphWalk::subgraphs`, `subgraph_nodes`. The `Labeller`/`GraphWalk` traits
  gained a subgraph type parameter which defaults to `()`, so existing
  implementations continue to work unchanged.
- `RankDir::as_str()`.

### Fixed

- Restored `Style::Invisible` (rendered as `style="invis"`), which was present
  in `0.1.4` but got lost during the subgraph merge.
- Cleaned up all `clippy` and `rustdoc` warnings; the crate now builds
  warning-free.

[#8]: https://github.com/przygienda/dot-rust/issues/8
[#16]: https://github.com/przygienda/dot-rust/pull/16

## 0.1.4

- Added invisible style.
- Various fixes; see the git history.
