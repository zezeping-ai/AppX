//! Windows UI Automation：读取焦点控件光标前文本。

use windows::core::BSTR;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
    IUIAutomationTextRange, IUIAutomationTextRangeArray, UIA_TextPatternId,
};

pub fn read_text_before_cursor(max_len: usize) -> Option<String> {
    if max_len == 0 {
        return Some(String::new());
    }

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
        let focused = automation.GetFocusedElement().ok()?;
        read_from_element(&focused, max_len)
    }
}

unsafe fn read_from_element(element: &IUIAutomationElement, max_len: usize) -> Option<String> {
    if let Some(text) = read_via_text_pattern(element, max_len) {
        return Some(text);
    }
    read_via_value(element, max_len)
}

unsafe fn read_via_text_pattern(
    element: &IUIAutomationElement,
    max_len: usize,
) -> Option<String> {
    let pattern = element
        .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
        .ok()?;
    let doc = pattern.DocumentRange().ok()?;
    let full = bstr_to_string(&doc.GetText(-1).ok()?)?;
    let caret = text_caret_offset(&pattern, &doc)?;
    let before = take_suffix_chars(&full, caret, max_len);
    Some(before)
}

unsafe fn text_caret_offset(
    pattern: &IUIAutomationTextPattern,
    doc: &IUIAutomationTextRange,
) -> Option<usize> {
    let selection: IUIAutomationTextRangeArray = pattern.GetSelection().ok()?;
    let len = selection.Length().ok()?;
    if len == 0 {
        return None;
    }
    let range = selection.GetElement(0).ok()?;
    let start = range.CompareEndpoints(
        windows::Win32::UI::Accessibility::TextPatternRangeEndpoint_End,
        doc,
        windows::Win32::UI::Accessibility::TextPatternRangeEndpoint_Start,
    ).ok()?;
    if start < 0 {
        return None;
    }
    Some(start as usize)
}

unsafe fn read_via_value(element: &IUIAutomationElement, max_len: usize) -> Option<String> {
    let value = element.GetCurrentPropertyValue(
        windows::Win32::UI::Accessibility::UIA_ValueValuePropertyId,
    ).ok()?;
    let full = value.to_string();
    if full.is_empty() {
        return None;
    }
    // 无 TextPattern 时无法获知光标，退化为取末尾窗口（部分控件仍可用）。
    Some(take_suffix_chars(&full, full.chars().count(), max_len))
}

fn bstr_to_string(value: &BSTR) -> Option<String> {
    Some(value.to_string())
}

fn take_suffix_chars(text: &str, caret: usize, max_len: usize) -> String {
    let caret = caret.min(text.chars().count());
    let before: String = text.chars().take(caret).collect();
    if before.chars().count() <= max_len {
        return before;
    }
    before
        .chars()
        .skip(before.chars().count().saturating_sub(max_len))
        .collect()
}
