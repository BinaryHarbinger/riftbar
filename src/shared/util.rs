#[inline]
pub fn take_chars(s: &str, x: u64) -> &str {
    if x == 0 {
        return "";
    }

    let mut count = 0;

    for (byte_idx, _) in s.char_indices() {
        if count == x {
            return &s[..byte_idx];
        }
        count += 1;
    }

    s
}
