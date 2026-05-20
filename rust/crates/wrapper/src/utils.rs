use jab_sys::wchar_t;

#[inline]
pub fn utf16_to_string(slice: &[wchar_t]) -> String {
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..end])
}
