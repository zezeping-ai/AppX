//! macOS Accessibility 公共封装（Dock 几何、焦点文本回溯等共用）。

use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::string::CFString;
use std::ffi::c_void;
use std::mem::MaybeUninit;

pub type AxElementRef = *mut c_void;
pub type AxError = i32;

pub const AX_ERROR_SUCCESS: AxError = 0;
pub const AX_VALUE_CG_POINT: i32 = 1;
pub const AX_VALUE_CG_SIZE: i32 = 2;
pub const AX_VALUE_CF_RANGE: i32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CFRange {
    pub location: isize,
    pub length: isize,
}

/// `AXUIElementCopyAttributeValue` 返回 +1 retain，需且仅需释放一次。
pub struct RetainedCf(pub *const c_void);

impl RetainedCf {
    pub fn into_ax_element(self) -> Option<AxElement> {
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

pub struct AxElement(AxElementRef);

impl AxElement {
    pub fn from_raw(ptr: AxElementRef) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> AxElementRef {
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
    fn AXUIElementCreateApplication(pid: i32) -> AxElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AxElementRef,
        attribute: *const c_void,
        value: *mut *const c_void,
    ) -> AxError;
    fn AXValueGetValue(value: *const c_void, value_type: i32, value_out: *mut c_void) -> bool;
}

pub fn system_wide() -> Option<AxElement> {
    AxElement::from_raw(unsafe { AXUIElementCreateSystemWide() })
}

pub fn application(pid: i32) -> Option<AxElement> {
    if pid <= 0 {
        return None;
    }
    AxElement::from_raw(unsafe { AXUIElementCreateApplication(pid) })
}

pub fn copy_attr(element: AxElementRef, name: &str) -> Option<RetainedCf> {
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

pub fn value_as_point(value: *const c_void) -> Option<CGPoint> {
    let mut point = MaybeUninit::<CGPoint>::uninit();
    let ok = unsafe { AXValueGetValue(value, AX_VALUE_CG_POINT, point.as_mut_ptr() as *mut c_void) };
    if ok {
        Some(unsafe { point.assume_init() })
    } else {
        None
    }
}

pub fn value_as_size(value: *const c_void) -> Option<CGSize> {
    let mut size = MaybeUninit::<CGSize>::uninit();
    let ok = unsafe { AXValueGetValue(value, AX_VALUE_CG_SIZE, size.as_mut_ptr() as *mut c_void) };
    if ok {
        Some(unsafe { size.assume_init() })
    } else {
        None
    }
}

pub fn value_as_range(value: *const c_void) -> Option<CFRange> {
    let mut range = MaybeUninit::<CFRange>::uninit();
    let ok = unsafe { AXValueGetValue(value, AX_VALUE_CF_RANGE, range.as_mut_ptr() as *mut c_void) };
    if ok {
        Some(unsafe { range.assume_init() })
    } else {
        None
    }
}

pub fn element_frame(element: AxElementRef) -> Option<(CGPoint, CGSize)> {
    let pos = copy_attr(element, "AXPosition")?;
    let size = copy_attr(element, "AXSize")?;
    let point = value_as_point(pos.0)?;
    let size = value_as_size(size.0)?;
    Some((point, size))
}

pub fn cfstring_from_retained(attr: RetainedCf) -> Option<String> {
    let ptr = attr.0;
    std::mem::forget(attr);
    if ptr.is_null() {
        return None;
    }
    let cf = unsafe { CFString::wrap_under_create_rule(ptr as _) };
    Some(cf.to_string())
}
