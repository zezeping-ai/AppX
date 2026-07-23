use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(not(target_os = "macos"))]
use tauri::Monitor;

use super::settings::{normalize_palette_layout, ClipboardAssistantSettings};

const GEOMETRY_FILE: &str = "palette-geometry.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaletteGeometry {
    /// 保存位置时对应的布局；与当前布局不一致时忽略坐标，避免切换布局后仍用旧尺寸。
    pub layout: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Copy)]
pub struct PaletteRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(not(target_os = "macos"))]
pub fn read_geometry(app: &AppHandle) -> Result<PaletteGeometry, String> {
    Ok(crate::app::settings::storage::read_json_settings(app, GEOMETRY_FILE, "浮层位置")?.unwrap_or_default())
}

pub fn write_geometry(app: &AppHandle, geometry: &PaletteGeometry) -> Result<(), String> {
    let mut geometry = geometry.clone();
    if let Ok(settings) = super::settings::read_settings(app) {
        sanitize_geometry(&settings, &mut geometry);
    }
    crate::app::settings::storage::write_json_settings(app, GEOMETRY_FILE, &geometry, "浮层位置")
}

const MIN_PANEL_HEIGHT: f64 = 300.0;
/// 侧栏条目内容宽 280px，窗口含内边距与滚动条约 320px
const SIDE_PANEL_WINDOW_WIDTH: f64 = 320.0;

fn layout_key(settings: &ClipboardAssistantSettings) -> String {
    normalize_palette_layout(&settings.palette_layout)
}

/// 创建浮层窗口时的默认内容高度。
pub fn inner_height_for_settings(settings: &ClipboardAssistantSettings) -> f64 {
    panel_height_for_layout(&layout_key(settings), settings)
}

fn panel_height_for_layout(layout: &str, settings: &ClipboardAssistantSettings) -> f64 {
    let panel_h = settings.palette_height.max(MIN_PANEL_HEIGHT as u32) as f64;
    match layout {
        "leftPanel" | "rightPanel" => 400.0,
        _ => panel_h,
    }
}

fn is_horizontal_panel(layout: &str) -> bool {
    matches!(layout, "topPanel" | "bottomPanel")
}

fn is_side_layout(layout: &str) -> bool {
    matches!(layout, "leftPanel" | "rightPanel")
}

fn side_panel_width(_settings: &ClipboardAssistantSettings) -> f64 {
    SIDE_PANEL_WINDOW_WIDTH
}

fn panel_height(settings: &ClipboardAssistantSettings) -> f64 {
    panel_height_for_layout(&layout_key(settings), settings)
}

pub fn invalidate_geometry_for_layout(app: &AppHandle, layout: &str) -> Result<(), String> {
    write_geometry(
        app,
        &PaletteGeometry {
            layout: Some(normalize_palette_layout(layout)),
            ..Default::default()
        },
    )
}

fn sanitize_geometry(settings: &ClipboardAssistantSettings, geometry: &mut PaletteGeometry) {
    let layout = layout_key(settings);
    geometry.layout.get_or_insert_with(|| layout.clone());
    if is_horizontal_panel(&layout) {
        geometry.height = Some(panel_height(settings));
        return;
    }
    if is_side_layout(&layout) {
        geometry.width = Some(side_panel_width(settings));
    }
}

pub fn capture_from_window(
    window: &tauri::WebviewWindow,
    layout: &str,
) -> Result<PaletteGeometry, String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    Ok(PaletteGeometry {
        layout: Some(normalize_palette_layout(layout)),
        x: Some(pos.x as f64 / scale),
        y: Some(pos.y as f64 / scale),
        width: Some(size.width as f64 / scale),
        height: Some(size.height as f64 / scale),
    })
}

// macOS 剪切面板用 Cocoa visible/Dock 定位；下列 Tauri Monitor 几何仅非 macOS 使用。

#[cfg(not(target_os = "macos"))]
fn work_area_logical(monitor: &Monitor) -> (f64, f64, f64, f64) {
    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    (
        work.position.x as f64 / scale,
        work.position.y as f64 / scale,
        work.size.width as f64 / scale,
        work.size.height as f64 / scale,
    )
}

#[cfg(not(target_os = "macos"))]
fn is_top_panel(layout: &str) -> bool {
    layout == "topPanel"
}

#[cfg(not(target_os = "macos"))]
fn edge_margin(_settings: &ClipboardAssistantSettings) -> f64 {
    0.0
}

#[cfg(not(target_os = "macos"))]
fn compute_side_rect(
    settings: &ClipboardAssistantSettings,
    monitor: &Monitor,
    right: bool,
) -> PaletteRect {
    let (wa_x, wa_y, wa_w, wa_h) = work_area_logical(monitor);
    let margin = edge_margin(settings);
    let width = side_panel_width(settings);
    PaletteRect {
        x: if right {
            wa_x + wa_w - width - margin
        } else {
            wa_x + margin
        },
        y: wa_y + margin,
        width,
        height: wa_h - margin * 2.0,
    }
}

#[cfg(not(target_os = "macos"))]
fn horizontal_panel_span(monitor: &Monitor) -> (f64, f64) {
    let (wa_x, _, wa_w, _) = work_area_logical(monitor);
    (wa_x, wa_w)
}

#[cfg(not(target_os = "macos"))]
fn bottom_panel_y(monitor: &Monitor, settings: &ClipboardAssistantSettings, height: f64) -> f64 {
    let (_, wa_y, _, wa_h) = work_area_logical(monitor);
    let margin = edge_margin(settings);
    wa_y + wa_h - height - margin
}

#[cfg(not(target_os = "macos"))]
fn top_panel_y(monitor: &Monitor, settings: &ClipboardAssistantSettings) -> f64 {
    let (_, wa_y, _, _) = work_area_logical(monitor);
    let margin = edge_margin(settings);
    wa_y + margin
}

#[cfg(not(target_os = "macos"))]
fn geometry_matches_layout(geometry: &PaletteGeometry, layout: &str) -> bool {
    geometry.layout.as_deref() == Some(layout)
}

#[cfg(not(target_os = "macos"))]
fn clamp_rect_to_work_area(monitor: &Monitor, rect: PaletteRect) -> PaletteRect {
    let (wa_x, wa_y, wa_w, wa_h) = work_area_logical(monitor);
    let width = rect.width.min(wa_w);
    let height = rect.height.min(wa_h);
    let x = rect.x.max(wa_x).min((wa_x + wa_w - width).max(wa_x));
    let y = rect.y.max(wa_y).min((wa_y + wa_h - height).max(wa_y));
    PaletteRect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(not(target_os = "macos"))]
fn resolve_horizontal_panel_rect(
    monitor: &Monitor,
    settings: &ClipboardAssistantSettings,
    layout: &str,
) -> PaletteRect {
    let height = panel_height(settings);
    let (x, width) = horizontal_panel_span(monitor);
    let y = if is_top_panel(layout) {
        top_panel_y(monitor, settings)
    } else {
        bottom_panel_y(monitor, settings, height)
    };
    clamp_rect_to_work_area(
        monitor,
        PaletteRect {
            x,
            y,
            width,
            height,
        },
    )
}

#[cfg(not(target_os = "macos"))]
fn resolve_rect(
    monitor: &Monitor,
    settings: &ClipboardAssistantSettings,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> PaletteRect {
    let layout = layout_key(settings);
    if is_horizontal_panel(&layout) {
        return resolve_horizontal_panel_rect(monitor, settings, &layout);
    }
    if is_side_layout(&layout) {
        let right = layout == "rightPanel";
        return clamp_rect_to_work_area(monitor, compute_side_rect(settings, monitor, right));
    }
    let height = height.max(panel_height_for_layout(&layout, settings));
    clamp_rect_to_work_area(
        monitor,
        PaletteRect {
            x,
            y,
            width,
            height,
        },
    )
}

/// 按布局计算默认窗口矩形（相对当前显示器 work area，避开 Dock / 菜单栏）。
#[cfg(not(target_os = "macos"))]
pub fn compute_rect(settings: &ClipboardAssistantSettings, monitor: &Monitor) -> PaletteRect {
    let (wa_x, wa_y, wa_w, wa_h) = work_area_logical(monitor);
    let margin = edge_margin(settings);
    let panel_h = settings.palette_height.max(MIN_PANEL_HEIGHT as u32) as f64;

    match layout_key(settings).as_str() {
        "rightPanel" => compute_side_rect(settings, monitor, true),
        "leftPanel" => compute_side_rect(settings, monitor, false),
        "topPanel" => PaletteRect {
            x: wa_x,
            y: wa_y + margin,
            width: wa_w,
            height: panel_h,
        },
        "bottomPanel" => PaletteRect {
            x: wa_x,
            y: wa_y + wa_h - panel_h - margin,
            width: wa_w,
            height: panel_h,
        },
        _ => PaletteRect {
            x: wa_x,
            y: wa_y + wa_h - panel_h - margin,
            width: wa_w,
            height: panel_h,
        },
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply_to_window(
    window: &tauri::WebviewWindow,
    app: &AppHandle,
    settings: &ClipboardAssistantSettings,
) -> Result<(), String> {
    let geometry = read_geometry(app)?;
    let layout = layout_key(settings);
    if settings.remember_window_position {
        if let (Some(x), Some(y), Some(w), Some(h)) =
            (geometry.x, geometry.y, geometry.width, geometry.height)
        {
            if geometry_matches_layout(&geometry, &layout) {
                let height = if is_horizontal_panel(&layout) {
                    panel_height(settings)
                } else {
                    h.max(panel_height_for_layout(&layout, settings))
                };
                let monitor = window
                    .current_monitor()
                    .map_err(|e| e.to_string())?
                    .or_else(|| window.primary_monitor().ok().flatten());
                let rect = if let Some(monitor) = &monitor {
                    resolve_rect(monitor, settings, x, y, w, height)
                } else {
                    PaletteRect {
                        x,
                        y,
                        width: w,
                        height,
                    }
                };
                window
                    .set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                        rect.width,
                        rect.height,
                    )))
                    .map_err(|e| e.to_string())?;
                window
                    .set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
                        rect.x, rect.y,
                    )))
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }

    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let rect = compute_rect(settings, &monitor);
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(rect.width, rect.height)))
        .map_err(|e| e.to_string())?;
    window
        .set_position(tauri::Position::Logical(tauri::LogicalPosition::new(rect.x, rect.y)))
        .map_err(|e| e.to_string())?;
    Ok(())
}
