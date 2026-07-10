//! macOS：CGEventTap 监听全局按键，仅用 keycode/flags，避免 TSM 主线程断言。

use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::thread;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CallbackResult, CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};
use foreign_types::ForeignType;
use tauri::AppHandle;

use super::{foreground, handle_keydown, handle_text_char, set_listener_active, ListenerState};

thread_local! {
    /// 专用监听线程内保存 tap mach port，用于超时后重载
    static TAP_MACH_PORT: Cell<CFMachPortRef> = const { Cell::new(std::ptr::null_mut()) };
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventKeyboardGetUnicodeString(
        event: core_graphics::sys::CGEventRef,
        max_string_len: u64,
        actual_string_len: *mut u64,
        unicode_string: *mut u16,
    );
}

pub fn start_listener(app: AppHandle) -> Result<(), String> {
    let own_bundle_id = app.config().identifier.clone();
    let state = Arc::new(Mutex::new(ListenerState::default()));
    thread::Builder::new()
        .name("abbrev-expansion".into())
        .spawn(move || {
            if let Err(err) = run_event_tap(app, own_bundle_id, state) {
                eprintln!("[code_snippets] CGEventTap failed: {err}");
            }
        })
        .map_err(|err| format!("spawn abbreviation listener: {err}"))?;
    Ok(())
}

fn run_event_tap(
    app: AppHandle,
    own_bundle_id: String,
    state: Arc<Mutex<ListenerState>>,
) -> Result<(), String> {
    let tap = unsafe {
        CGEventTap::new_unchecked(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            // 仅 KeyDown 参与 mask；TapDisabled* 值过大会导致 CGEventMaskBit 移位溢出，
            // 但禁用通知仍会投递到 callback，可在下方分支里 re-enable。
            vec![CGEventType::KeyDown],
            {
                let app = app.clone();
                let state = state.clone();
                let own_bundle_id = own_bundle_id.clone();
                move |_proxy, event_type, event| {
                    match event_type {
                        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                            reenable_tap();
                            return CallbackResult::Keep;
                        }
                        CGEventType::KeyDown => {}
                        _ => return CallbackResult::Keep,
                    }

                    if event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0 {
                        return CallbackResult::Keep;
                    }

                    if foreground::is_own_app_foreground(&own_bundle_id) {
                        return CallbackResult::Keep;
                    }

                    let flags = event.get_flags();
                    if flags.intersects(
                        CGEventFlags::CGEventFlagCommand
                            | CGEventFlags::CGEventFlagControl
                            | CGEventFlags::CGEventFlagAlternate,
                    ) {
                        return CallbackResult::Keep;
                    }

                    let shift_held = flags.contains(CGEventFlags::CGEventFlagShift);
                    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

                    if let Some(ch) = event_text_char(&event) {
                        handle_text_char(&app, &state, ch);
                    } else {
                        handle_keydown(&app, &state, shift_held, keycode);
                    }

                    CallbackResult::Keep
                }
            },
        )
    }
    .map_err(|()| "create CGEventTap (grant Accessibility permission)".to_string())?;

    TAP_MACH_PORT.with(|port| {
        port.set(tap.mach_port().as_concrete_TypeRef());
    });

    let source = tap
        .mach_port()
        .create_runloop_source(0)
        .map_err(|_| "create runloop source".to_string())?;

    unsafe {
        CFRunLoop::get_current().add_source(&source, kCFRunLoopCommonModes);
    }
    tap.enable();
    set_listener_active(true);
    log::info!("[code_snippets] CGEventTap listener active");

    // 保持 tap 存活直至 run loop 结束
    let _tap = tap;
    CFRunLoop::run_current();

    TAP_MACH_PORT.with(|port| port.set(std::ptr::null_mut()));
    set_listener_active(false);
    Ok(())
}

fn event_text_char(event: &CGEvent) -> Option<char> {
    let mut buf = [0u16; 4];
    let mut len = 0u64;
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event.as_ptr(),
            buf.len() as u64,
            &mut len,
            buf.as_mut_ptr(),
        );
    }
    if len == 0 {
        return None;
    }
    let slice = &buf[..len.min(buf.len() as u64) as usize];
    let text = String::from_utf16_lossy(slice);
    let mut chars = text.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(ch)
}

fn reenable_tap() {
    TAP_MACH_PORT.with(|port| {
        let mach = port.get();
        if mach.is_null() {
            log::warn!("[code_snippets] CGEventTap disabled but port not set");
            return;
        }
        unsafe { CGEventTapEnable(mach, true) };
    });
}
