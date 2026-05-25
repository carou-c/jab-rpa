use std::slice;

use jab_sys::wchar_t;

#[inline]
pub fn utf16_to_string(slice: &[wchar_t]) -> String {
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..end])
}

#[inline]
/// # Safety
///
/// The same guarantees of std::slice::from_raw_parts must be upheld,
/// except non-nullness.
///
/// A null pointer is transformed into an empty string
pub unsafe fn raw_utf16_to_string(ptr: *mut wchar_t, len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..end])
}
