use glam::Mat4;
use miracle_plugin::{
    core::{Point, Rectangle, Size},
    input::{InputEventModifiers, KeyboardAction, KeyboardEvent},
    miracle_plugin,
    placement::{FreestylePlacement, Placement},
    plugin::{Plugin, get_active_workspace, get_userdata_json, managed_windows},
    window::{DepthLayer, WindowInfo, WindowState, WindowType},
    workspace::Workspace,
};
use std::collections::HashMap;

const WINDOW_WIDTH_FRACTION: f32 = 0.5;
const DEFAULT_INNER_GAP: i32 = 20;
const DEFAULT_OUTER_GAP: i32 = 0;

const XKB_KEY_LEFT: u32 = 0xff51;
const XKB_KEY_RIGHT: u32 = 0xff53;

struct Miri {
    /// Per-workspace ordered list of managed Normal windows.
    workspaces: HashMap<u64, MiriWorkspaceInfo>,
    inner_gap: i32,
    outer_gap: i32,
    workspace: Option<u32>,
}

impl Default for Miri {
    fn default() -> Self {
        let (inner_gap, outer_gap, workspace) = Self::load_config();
        Self {
            workspaces: HashMap::new(),
            inner_gap,
            outer_gap,
            workspace: if workspace < 0 {
                None
            } else {
                Some(workspace as u32)
            },
        }
    }
}

struct MiriWorkspaceInfo {
    windows: Vec<u64>,
    /// Index of the currently focused window (used for keyboard navigation).
    focused_index: usize,
    /// Index of the leftmost visible window (the scroll/viewport position).
    viewport_index: usize,
    /// The usable area of the workspace (updated via workspace_area_changed).
    area: Rectangle,
}

impl Miri {
    fn load_config() -> (i32, i32, i32) {
        let json = match get_userdata_json() {
            Some(j) => j,
            None => return (DEFAULT_INNER_GAP, DEFAULT_OUTER_GAP, -1),
        };
        let v: serde_json::Value = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(_) => return (DEFAULT_INNER_GAP, DEFAULT_OUTER_GAP, -1),
        };
        let inner_gap = v["inner_gap"].as_i64().unwrap_or(DEFAULT_INNER_GAP as i64) as i32;
        let outer_gap = v["outer_gap"].as_i64().unwrap_or(DEFAULT_OUTER_GAP as i64) as i32;
        let workspace = v["workspace"].as_i64().unwrap_or(-1) as i32;
        (inner_gap, outer_gap, workspace)
    }

    fn effective_area(area: &Rectangle, outer_gap: i32) -> Rectangle {
        Rectangle {
            x: area.x + outer_gap,
            y: area.y + outer_gap,
            width: (area.width - 2 * outer_gap).max(0),
            height: (area.height - 2 * outer_gap).max(0),
        }
    }

    fn window_width(effective: &Rectangle) -> i32 {
        (effective.width as f32 * WINDOW_WIDTH_FRACTION) as i32
    }

    fn stride(effective: &Rectangle, inner_gap: i32) -> i32 {
        Self::window_width(effective) + inner_gap
    }

    /// The rectangle for a window at `index` given the viewport anchor for that workspace.
    /// The window at `viewport_index` is anchored at the workspace's top-left; others are placed side-by-side.
    fn rect_for_index(
        area: &Rectangle,
        index: usize,
        viewport_index: usize,
        inner_gap: i32,
        outer_gap: i32,
    ) -> Rectangle {
        let effective = Self::effective_area(area, outer_gap);
        let x = effective.x
            + (index as i32 - viewport_index as i32) * Self::stride(&effective, inner_gap);
        Rectangle {
            x,
            y: effective.y,
            width: Self::window_width(&effective),
            height: effective.height,
        }
    }

    /// Find which workspace and index a window belongs to.
    fn find_window(&self, win_info: &WindowInfo) -> Option<(u64, usize)> {
        for (ws_id, info) in &self.workspaces {
            if let Some(idx) = info.windows.iter().position(|w| *w == win_info.id()) {
                return Some((*ws_id, idx));
            }
        }
        None
    }

    fn relayout(&self, ws_id: u64, animate: bool) {
        let info = match self.workspaces.get(&ws_id) {
            Some(w) => w,
            None => return,
        };
        if info.windows.is_empty() {
            return;
        }
        let managed = managed_windows();
        for (index, stored_info) in info.windows.iter().enumerate() {
            if let Some(pw) = managed.iter().find(|pw| {
                let wi: &WindowInfo = pw;
                wi.id() == *stored_info
            }) {
                let _ = pw.set_rectangle(
                    Self::rect_for_index(
                        &info.area,
                        index,
                        info.viewport_index,
                        self.inner_gap,
                        self.outer_gap,
                    ),
                    animate,
                );
            }
        }
    }

    fn focus_window_at(&self, ws_id: u64, index: usize) {
        let managed = managed_windows();
        if let Some(info) = self.workspaces.get(&ws_id) {
            if let Some(target) = info.windows.get(index) {
                if let Some(pw) = managed.iter().find(|pw| {
                    let wi: &WindowInfo = pw;
                    wi.id() == *target
                }) {
                    let _ = pw.request_focus();
                }
            }
        }
    }

    fn is_on_required_workspace(&self) -> bool {
        if let Some(required_workspace_num) = self.workspace {
            if let Some(active) = get_active_workspace() {
                if let Some(num) = active.number {
                    if num != required_workspace_num {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Update `viewport_index` to minimally scroll so the window at `idx` is visible.
    fn scroll_into_view_if_needed(
        ws_info: &mut MiriWorkspaceInfo,
        idx: usize,
        inner_gap: i32,
        outer_gap: i32,
    ) {
        let effective = Self::effective_area(&ws_info.area, outer_gap);
        let stride = Self::stride(&effective, inner_gap);
        let offset = (idx as i32 - ws_info.viewport_index as i32) * stride;

        if offset < 0 {
            // Off-screen to the left: bring it to position 0.
            ws_info.viewport_index = idx;
        } else if stride > 0 && offset >= effective.width {
            // Off-screen to the right: scroll minimally so idx is the last visible window.
            let slots = ((effective.width - 1) / stride) as usize;
            ws_info.viewport_index = idx.saturating_sub(slots);
        }
        // else: already visible, no scroll needed.
    }
}

impl Plugin for Miri {
    fn place_new_window(&mut self, info: &WindowInfo) -> Option<Placement> {
        if !self.is_on_required_workspace() {
            return None;
        }

        if info.window_type != WindowType::Normal && info.window_type != WindowType::Freestyle {
            return None;
        }

        if info.state == WindowState::Attached {
            return None;
        }

        let ws = get_active_workspace()?;
        let ws_id = ws.id();

        let workspace_info = self.workspaces.get_mut(&ws_id).unwrap();
        let new_index = workspace_info.windows.len();
        let viewport_index = workspace_info.viewport_index;
        let rect = Self::rect_for_index(
            &workspace_info.area,
            new_index,
            viewport_index,
            self.inner_gap,
            self.outer_gap,
        );
        workspace_info.windows.push(info.id());

        // Place at the natural next slot. window_focused will fire immediately after
        // and call relayout() to scroll all windows into their correct positions.
        Some(Placement::Freestyle(FreestylePlacement {
            top_left: Point::new(rect.x, rect.y),
            depth_layer: DepthLayer::Application,
            workspace: Some(ws),
            size: Size::new(rect.width, rect.height),
            transform: Mat4::IDENTITY,
            alpha: 1.0,
            movable: false,
            resizable: false,
        }))
    }

    fn window_deleted(&mut self, info: &WindowInfo) {
        if let Some((ws_id, idx)) = self.find_window(&info) {
            let ws_info = self.workspaces.get_mut(&ws_id).unwrap();
            ws_info.windows.remove(idx);

            if !ws_info.windows.is_empty() {
                // If we removed at or before the focused/viewport window, pull the indices back.
                if idx <= ws_info.focused_index && ws_info.focused_index > 0 {
                    ws_info.focused_index -= 1;
                }
                ws_info.focused_index = ws_info.focused_index.min(ws_info.windows.len() - 1);

                if idx <= ws_info.viewport_index && ws_info.viewport_index > 0 {
                    ws_info.viewport_index -= 1;
                }
                ws_info.viewport_index = ws_info.viewport_index.min(ws_info.windows.len() - 1);

                self.relayout(ws_id, true);
            } else {
                ws_info.focused_index = 0;
                ws_info.viewport_index = 0;
            }
        }
    }

    fn window_focused(&mut self, info: &WindowInfo) {
        if let Some((ws_id, idx)) = self.find_window(&info) {
            if let Some(ws_info) = self.workspaces.get_mut(&ws_id) {
                // Always track which window is focused (used by keyboard navigation).
                ws_info.focused_index = idx;
                // Only scroll the viewport if the window isn't already visible.
                Self::scroll_into_view_if_needed(ws_info, idx, self.inner_gap, self.outer_gap);
            }
            self.relayout(ws_id, true);
        }
    }

    fn workspace_created(&mut self, workspace: &Workspace) {
        self.workspaces
            .entry(workspace.id())
            .or_insert(MiriWorkspaceInfo {
                windows: vec![],
                focused_index: 0,
                viewport_index: 0,
                area: workspace.rectangle,
            });
    }

    fn workspace_removed(&mut self, workspace: &Workspace) {
        self.workspaces.remove(&workspace.id());
    }

    fn workspace_focused(&mut self, _previous_id: Option<u64>, current: &Workspace) {
        let ws_id = current.id();
        if self
            .workspaces
            .get(&ws_id)
            .map_or(false, |w| !w.windows.is_empty())
        {
            self.focus_window_at(ws_id, 0);
        }
    }

    fn workspace_area_changed(&mut self, workspace: &Workspace) {
        let ws_id = workspace.id();
        // Create the entry if it doesn't exist yet so the correct area is never
        // discarded when this callback fires before workspace_created.
        self.workspaces
            .entry(ws_id)
            .or_insert_with(|| MiriWorkspaceInfo {
                windows: vec![],
                focused_index: 0,
                viewport_index: 0,
                area: workspace.rectangle,
            })
            .area = workspace.rectangle;
        self.relayout(ws_id, false);
    }

    fn handle_keyboard_input(&mut self, event: KeyboardEvent) -> bool {
        if !self.is_on_required_workspace() {
            return false;
        }

        if event.action != KeyboardAction::Down {
            return false;
        }

        if !event.modifiers.contains(InputEventModifiers::META) {
            return false;
        }

        let ws_id = match get_active_workspace() {
            Some(ws) => ws.id(),
            None => return false,
        };

        let info = match self.workspaces.get(&ws_id) {
            Some(i) => i,
            None => return false,
        };

        match event.keysym {
            XKB_KEY_LEFT if info.focused_index > 0 => {
                self.focus_window_at(ws_id, info.focused_index - 1);
                true
            }
            XKB_KEY_RIGHT if info.focused_index + 1 < info.windows.len() => {
                self.focus_window_at(ws_id, info.focused_index + 1);
                true
            }
            XKB_KEY_LEFT | XKB_KEY_RIGHT => true, // consume even at boundary
            _ => false,
        }
    }
}

miracle_plugin!(Miri);
