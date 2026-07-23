//! macOS Accessibility：读取焦点输入框光标前文本，供 F12 回溯解析缩写。

use crate::app::platform::macos_ax::{
    copy_attr, cfstring_from_retained, system_wide, value_as_range,
};

/// 读取焦点控件光标前最多 `max_len` 个字符（F12 KeyDown 时不写入文档）。
pub fn read_text_before_cursor(max_len: usize) -> Option<String> {
    if max_len == 0 {
        return Some(String::new());
    }

    let system = system_wide()?;
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

fn selected_range_location(element: crate::app::platform::macos_ax::AxElementRef) -> Option<usize> {
    let range_attr = copy_attr(element, "AXSelectedTextRange")?;
    let range = value_as_range(range_attr.0)?;
    if range.location < 0 {
        return None;
    }
    Some(range.location as usize)
}

fn take_suffix_chars(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    text.chars()
        .skip(text.chars().count().saturating_sub(max_len))
        .collect()
}
