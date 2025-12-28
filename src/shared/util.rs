#[inline]
pub fn take_chars(s: &str, x: u64) -> &str {
    if x == 0 {
        return "";
    }

    for (count, (byte_idx, _)) in s.char_indices().enumerate() {
        if count as u64 == x {
            return &s[..byte_idx];
        }
    }

    s
}
