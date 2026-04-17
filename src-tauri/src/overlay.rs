//! macOS overlay backend.
//! The Windows port used raw Win32 APIs (SetWindowLongW, DwmSetWindowAttribute, etc.).
//! On macOS we rely entirely on Tauri's cross-platform window API — no native FFI needed.

use crate::app_state::ClickerState;
use crate::engine::mouse::{current_monitor_rects, current_virtual_screen_rect, VirtualScreenRect};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

static LAST_ZONE_SHOW: Mutex<Option<Instant>> = Mutex::new(None);
pub static OVERLAY_THREAD_RUNNING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

pub fn init_overlay(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;

    log::info!("[Overlay] Running one-time init...");

    window
        .set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
    let _ = window.set_decorations(false);
    // Keep overlay above other windows without stealing focus.
    let _ = window.set_always_on_top(true);

    let bounds = current_virtual_screen_rect();
    if let Some(b) = bounds {
        let _ = window.set_position(tauri::PhysicalPosition::new(b.left, b.top));
        let _ = window.set_size(tauri::PhysicalSize::new(b.width as u32, b.height as u32));
    }

    // Start hidden — shown only when the clicker is active.
    let _ = window.hide();

    log::info!("[Overlay] Init complete — window configured but hidden");
    Ok(())
}

pub fn show_overlay(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ClickerState>();
    if !state.settings_initialized.load(Ordering::SeqCst) {
        return Ok(());
    }
    {
        let settings = state.settings.lock().unwrap();
        if !settings.show_stop_overlay {
            return Ok(());
        }
    }

    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;

    let bounds = current_virtual_screen_rect()
        .ok_or_else(|| "Virtual screen bounds not available".to_string())?;

    // Resize to cover the full virtual screen, then show.
    let _ = window.set_position(tauri::PhysicalPosition::new(bounds.left, bounds.top));
    let _ = window.set_size(tauri::PhysicalSize::new(bounds.width as u32, bounds.height as u32));

    let visible = window.is_visible().unwrap_or(false);
    if !visible {
        let _ = window.show();
    }

    *LAST_ZONE_SHOW.lock().unwrap() = Some(Instant::now());

    let settings = state.settings.lock().unwrap();
    let monitors = current_monitor_rects().unwrap_or_else(|| vec![bounds]);
    let monitor_payload: Vec<_> = monitors
        .into_iter()
        .map(|monitor| {
            let offset = monitor.offset_from(bounds);
            serde_json::json!({
                "x": offset.left,
                "y": offset.top,
                "width": offset.width,
                "height": offset.height,
            })
        })
        .collect();

    let _ = window.emit(
        "zone-data",
        serde_json::json!({
            "edgeStopEnabled": settings.edge_stop_enabled,
            "edgeStopTop": settings.edge_stop_top,
            "edgeStopRight": settings.edge_stop_right,
            "edgeStopBottom": settings.edge_stop_bottom,
            "edgeStopLeft": settings.edge_stop_left,
            "cornerStopEnabled": settings.corner_stop_enabled,
            "cornerStopTL": settings.corner_stop_tl,
            "cornerStopTR": settings.corner_stop_tr,
            "cornerStopBL": settings.corner_stop_bl,
            "cornerStopBR": settings.corner_stop_br,
            "screenWidth": bounds.width,
            "screenHeight": bounds.height,
            "monitors": monitor_payload,
            "_showDisabledEdges": !settings.edge_stop_enabled,
            "_showDisabledCorners": !settings.corner_stop_enabled,
        }),
    );

    Ok(())
}

pub fn check_auto_hide(app: &AppHandle) {
    let mut last = LAST_ZONE_SHOW.lock().unwrap();
    if let Some(instant) = *last {
        if instant.elapsed() >= Duration::from_secs(3) {
            *last = None;
            if let Some(window) = app.get_webview_window("overlay") {
                log::info!("[Overlay] Auto-hide: hiding window");
                let _ = window.hide();
            }
        }
    }
}

#[tauri::command]
pub fn hide_overlay(app: AppHandle) -> Result<(), String> {
    *LAST_ZONE_SHOW.lock().unwrap() = None;
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
    Ok(())
}
