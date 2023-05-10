//! # [egui](https://github.com/emilk/egui) hierarchial tile manager
//! Tiles that can be arranges in horizontal, vertical, and grid-layouts, or put in tabs.
//! The tiles can be resized and re-arranged by drag-and-drop.
//!
//! ## Overview
//! The fundamental unit is the [`Tile`] which is either a [`Container`] or a `Pane` (a leaf).
//! The [`Tile`]s are put into a [`Tree`].
//! Everything is generic over the type of panes, leaving up to the user what to store in the tree.
//!
//! Each [`Tile`] is identified by a (random) [`TileId`].
//! The tiles are stored in [`Tiles`].
//!
//! The entire state is stored in a single [`Tree`] struct which consists of a [`Tiles`] and a root [`TileId`].
//!
//! The behavior and the look of the [`Tree`] is controlled by the [`Behavior`] `trait`.
//! The user needs to implement this in order to specify the `ui` of each `Pane` and
//! the tab name of panes (if there are tab tiles).
//!
//! ## Shares
//! The relative sizes of linear layout (horizontal or vertical) and grid columns and rows are specified by _shares_.
//! If the shares are `1,2,3` it means the first element gets `1/6` of the space, the second `2/6`, and the third `3/6`.
//! The default share size is `1`, and when resizing the shares are restributed so that
//! the total shares are always approximately the same as the number of rows/columns.
//! This makes it easy to add new rows/columns.
//!
//! ## Shortcomings
//! The implementation is recursive, so if your trees get too deep you will get a stack overflow.
//!
//! ## Future improvements
//! * Easy per-tab close-buttons
//! * Scrolling of tab-bar
//! * Vertical tab bar

// ## Implementation notes
// In many places we want to recursively visit all tiles, while also mutating them.
// In order to not get into trouble with the borrow checker a trick is used:
// each [`Tile`] is removed, mutated, recursed, and then re-added.
// You'll see this pattern many times reading the following code.
//
// Each frame consists of two passes: layout, and ui.
// The layout pass figures out where each tile should be placed.
// The ui pass does all the painting.
// These two passes could be combined into one pass if we wanted to,
// but having them split up makes the code slightly simpler, and
// leaves the door open for more complex layout (e.g. min/max sizes per tile).
//
// Everything is quite dynamic, so we have a bunch of defensive coding that call `warn!` on failure.
// These situations should not happen in normal use, but could happen if the user messes with
// the internals of the tree, putting it in an invalid state.

#![forbid(unsafe_code)]

use egui::{Pos2, Rect};

mod behavior;
mod container;
mod tile;
mod tiles;
mod tree;

pub use behavior::Behavior;
pub use container::{Container, Grid, GridLoc, Layout, Linear, LinearDir, Tabs};
pub use tile::{Tile, TileId};
pub use tiles::Tiles;
pub use tree::Tree;

// ----------------------------------------------------------------------------

/// The response from [`Behavior::pane_ui`] for a pane.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiResponse {
    #[default]
    None,

    /// The viewer is being dragged via some element in the Pane
    DragStarted,
}

/// What are the rules for simplifying the tree?
///
/// Drag-dropping tiles can often leave containers empty, or with only a single child.
/// The [`SimplificationOptions`] specifies what simplifications are allowed.
///
/// The [`Tree`] will run a simplification pass each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimplificationOptions {
    pub prune_empty_tabs: bool,
    pub prune_single_child_tabs: bool,
    pub prune_empty_layouts: bool,
    pub prune_single_child_layouts: bool,
    pub all_panes_must_have_tabs: bool,
    /// If a horizontal layout contain another horizontal layout, join them?
    /// Same for vertical layouts. Does NOT apply to grid layout or tab layouts.
    pub join_nested_linear_layouts: bool,
}

impl Default for SimplificationOptions {
    fn default() -> Self {
        Self {
            prune_empty_tabs: true,
            prune_single_child_tabs: true,
            prune_empty_layouts: true,
            prune_single_child_layouts: true,
            all_panes_must_have_tabs: false,
            join_nested_linear_layouts: true,
        }
    }
}

/// The current state of a resize handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResizeState {
    Idle,

    /// The user is hovering over the resize handle.
    Hovering,

    /// The user is dragging the resize handle.
    Dragging,
}

// ----------------------------------------------------------------------------

/// An insertion point in a specific containter.
///
/// Specifies the expected container layout type, and where to insert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerInsertion {
    Tabs(usize),
    Horizontal(usize),
    Vertical(usize),
    Grid(GridLoc),
}

/// Where in the tree to insert a tile.
#[derive(Clone, Copy, Debug)]
struct InsertionPoint {
    pub parent_id: TileId,

    /// Where in the parent?
    pub insertion: ContainerInsertion,
}

impl InsertionPoint {
    pub fn new(parent_id: TileId, insertion: ContainerInsertion) -> Self {
        Self {
            parent_id,
            insertion,
        }
    }
}

#[derive(PartialEq, Eq)]
enum GcAction {
    Keep,
    Remove,
}

#[must_use]
enum SimplifyAction {
    Remove,
    Keep,
    Replace(TileId),
}

fn is_possible_drag(ctx: &egui::Context) -> bool {
    ctx.input(|input| input.pointer.is_decidedly_dragging())
}

fn is_being_dragged(ctx: &egui::Context, tile_id: TileId) -> bool {
    ctx.memory(|mem| mem.is_being_dragged(tile_id.id())) && is_possible_drag(ctx)
}

// ----------------------------------------------------------------------------

/// Context used for drag-and-dropping of tiles.
///
/// This is passed down during the `ui` pass.
/// Each tile registers itself with this context.
struct DropContext {
    enabled: bool,
    dragged_tile_id: Option<TileId>,
    mouse_pos: Option<Pos2>,

    best_insertion: Option<InsertionPoint>,
    best_dist_sq: f32,
    preview_rect: Option<Rect>,
}

impl DropContext {
    fn on_tile<Pane>(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        style: &egui::Style,
        parent_id: TileId,
        rect: Rect,
        tile: &Tile<Pane>,
    ) {
        if !self.enabled {
            return;
        }

        if tile.layout() != Some(Layout::Horizontal) {
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Horizontal(0)),
                rect.split_left_right_at_fraction(0.5).0,
            );
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Horizontal(usize::MAX)),
                rect.split_left_right_at_fraction(0.5).1,
            );
        }

        if tile.layout() != Some(Layout::Vertical) {
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Vertical(0)),
                rect.split_top_bottom_at_fraction(0.5).0,
            );
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Vertical(usize::MAX)),
                rect.split_top_bottom_at_fraction(0.5).1,
            );
        }

        self.suggest_rect(
            InsertionPoint::new(parent_id, ContainerInsertion::Tabs(usize::MAX)),
            rect.split_top_bottom_at_y(rect.top() + behavior.tab_bar_height(style))
                .1,
        );
    }

    fn suggest_rect(&mut self, insertion: InsertionPoint, preview_rect: Rect) {
        if !self.enabled {
            return;
        }
        let target_point = preview_rect.center();
        if let Some(mouse_pos) = self.mouse_pos {
            let dist_sq = mouse_pos.distance_sq(target_point);
            if dist_sq < self.best_dist_sq {
                self.best_dist_sq = dist_sq;
                self.best_insertion = Some(insertion);
                self.preview_rect = Some(preview_rect);
            }
        }
    }
}
