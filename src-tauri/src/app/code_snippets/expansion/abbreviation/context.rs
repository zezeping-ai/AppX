//! macOS Accessibility：读取焦点输入框光标前文本，供 F12 回溯解析缩写。

use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::string::CFString;
use std::ffi::c_void;
use std::mem::MaybeUninit;

type AxElementRef = *mut c_void;
type AxError = i32;

const AX_ERROR_SUCCESS: AxError = 0;
const AX_VALUE_CF_RANGE_TYPE: i32 = 4;

#[repr(C)]
#[derive(Clone, Copy)]
struct CFRange {
    location: isize,
    length: isize,
}

/// `AXUIElementCopyAttributeValue` 返回 +1 retain，需且仅需释放一次。
struct RetainedCf(*const c_void);

impl RetainedCf {
    fn into_ax_element(self) -> Option<AxElement> {
        let ptr = self.0 as AxElementRef;
        std::mem::forget(self);
        AxElement::from_raw(ptr)
    }
}

impl Drop for RetainedCf {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

struct AxElement(AxElementRef);

impl AxElement {
    fn from_raw(ptr: AxElementRef) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    fn as_ptr(&self) -> AxElementRef {
        self.0
    }
}

impl Drop for AxElement {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AxElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AxElementRef,
        attribute: *const c_void,
        value: *mut *const c_void,
    ) -> AxError;
    fn AXValueGetValue(value: *const c_void, value_type: i32, value_out: *mut c_void) -> bool;
}

/// 读取焦点控件光标前最多 `max_len` 个字符（F12 KeyDown 时不写入文档）。
pub fn read_text_before_cursor(max_len: usize) -> Option<String> {
    if max_len == 0 {
        return Some(String::new());
    }

    let system = AxElement::from_raw(unsafe { AXUIElementCreateSystemWide() })?;
    let focused = copy_attr(system.as_ptr(), "AXFocusedUIElement")?;
    let focused_el = focused.into_ax_element()?;

    let cursor = selected_range_location(focused_el.as_ptr())?;
    if cursor == 0 {
        return Some(String::new());
    }

    let value = copy_attr(focused_el.as_ptr(), "AXValue")?;
    let full_text = cfstring_from_retained(value)?;
    let cursor = cursor.min(full_text.chars().count());
    let before: String = full_text.chars().take(cursor).collect();
    Some(take_suffix_chars(&before, max_len))
}

fn copy_attr(element: AxElementRef, name: &str) -> Option<RetainedCf> {
    let key = CFString::new(name);
    let mut value: *const c_void = std::ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(
            element,
            key.as_concrete_TypeRef() as *const c_void,
            &mut value,
        )
    };
    if err != AX_ERROR_SUCCESS || value.is_null() {
        return None;
    }
    Some(RetainedCf(value))
}

fn selected_range_location(element: AxElementRef) -> Option<usize> {
    let range_attr = copy_attr(element, "AXSelectedTextRange")?;
    let mut range = MaybeUninit::<CFRange>::uninit();
    let ok = unsafe {
        AXValueGetValue(
            range_attr.0,
            AX_VALUE_CF_RANGE_TYPE,
            range.as_mut_ptr() as *mut c_void,
        )
    };
    if !ok {
        return None;
    }
    let range = unsafe { range.assume_init() };
    if range.location < 0 {
        return None;
    }
    Some(range.location as usize)
}

fn cfstring_from_retained(attr: RetainedCf) -> Option<String> {
    let ptr = attr.0;
    std::mem::forget(attr);
    if ptr.is_null() {
        return None;
    }
    let cf = unsafe { CFString::wrap_under_create_rule(ptr as _) };
    Some(cf.to_string())
}

fn take_suffix_chars(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    text.chars()
        .skip(text.chars().count().saturating_sub(max_len))
        .collect()
}
