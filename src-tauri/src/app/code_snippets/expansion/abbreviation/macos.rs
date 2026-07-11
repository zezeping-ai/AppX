//! macOS：CGEventTap 监听展开触发键，回溯焦点文本解析缩写。

use std::cell::Cell;
use std::thread;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CallbackResult, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};
use tauri::AppHandle;

use super::{foreground, is_expand_trigger_key, set_listener_active, try_expand_on_trigger};

thread_local! {
    static TAP_MACH_PORT: Cell<CFMachPortRef> = const { Cell::new(std::ptr::null_mut()) };
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

pub fn start_listener(app: AppHandle) -> Result<(), String> {
    let own_bundle_id = app.config().identifier.clone();
    thread::Builder::new()
        .name("abbrev-expansion".into())
        .spawn(move || {
            if let Err(err) = run_event_tap(app, own_bundle_id) {
                eprintln!("[code_snippets] CGEventTap failed: {err}");
            }
        })
        .map_err(|err| format!("spawn abbreviation listener: {err}"))?;
    Ok(())
}

fn run_event_tap(app: AppHandle, own_bundle_id: String) -> Result<(), String> {
    let tap = unsafe {
        CGEventTap::new_unchecked(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::KeyDown],
            {
                let app = app.clone();
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
                    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

                    if is_expand_trigger_key(keycode, flags) {
                        let app = app.clone();
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            try_expand_on_trigger(&app);
                        }));
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
    log::info!("[code_snippets] CGEventTap listener active (abbrev lookback trigger)");

    let _tap = tap;
    CFRunLoop::run_current();

    TAP_MACH_PORT.with(|port| port.set(std::ptr::null_mut()));
    set_listener_active(false);
    Ok(())
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
