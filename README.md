# `egui_tile_tree`

[<img alt="github" src="https://img.shields.io/badge/github-rerun-io/egui_tile_tree-8da0cb?logo=github" height="20">](https://github.com/rerun-io/egui_tile_tree)
[![Latest version](https://img.shields.io/crates/v/egui_tile_tree.svg)](https://crates.io/crates/egui_tile_tree)
[![Documentation](https://docs.rs/egui_tile_tree/badge.svg)](https://docs.rs/egui_tile_tree)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Build Status](https://github.com/rerun-io/egui_tile_tree/workflows/CI/badge.svg)](https://github.com/rerun-io/egui_tile_tree/actions?workflow=CI)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/rerun-io/egui_tile_tree/blob/master/LICENSE-MIT)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/rerun-io/egui_tile_tree/blob/master/LICENSE-APACHE)

Layouting and docking for [egui](https://github.com/rerun-io/egui).

Supports:
* Horizontal and vertical layouts
* Grid layouts
* Tabs
* Drag-and-drop docking

### Comparison with [egui_dock](https://github.com/Adanos020/egui_dock)
[egui_dock](https://github.com/Adanos020/egui_dock) is an excellent crate serving similar needs. `egui_tile_tree` aims to become a more flexible and feature-rich alternative to `egui_dock`.

`egui_dock` only supports binary splits (left/right or top/bottom), while `egui_tile_tree` support full horizontal and vertical layouts, as well as grid layouts. `egui_tile_tree` is also designed to be more flexible, enabling users to customize the behavior by implementing a `Behavior` `trait`.

`egui_dock` supports some features that `egui_tile_tree` does not yet support, such as close-buttons on each tab, and built-in scroll areas.

---

<div align="center">
<img src="media/rerun_io_logo.png" width="50%">

egui development is sponsored by [Rerun](https://www.rerun.io/), a startup doing<br>
visualizations for computer vision and robotics.
</div>
