// -----------------------------------------------------------------------------
// Hex Parsing
// -----------------------------------------------------------------------------

/// Parse a hex number into `usize`.
///
/// Accepts optional 0x/0X prefix and ignores surrounding whitespace.
pub(crate) fn parse_hex_usize(s: &str) -> Option<usize> {
    let s = trim_hex(s);
    if s.is_empty() {
        return None;
    }

    usize::from_str_radix(s, 16).ok()
}

/// Parse a single byte token (exactly two hex digits), optionally prefixed with 0x/0X.
pub(crate) fn parse_hex_u8_token(s: &str) -> Option<u8> {
    let s = trim_hex(s);
    if s.len() != 2 {
        return None;
    }

    u8::from_str_radix(s, 16).ok()
}

/// Convert a nibble into a lowercase ASCII hex digit.
pub(crate) fn hex_digit(n: u8) -> u8 {
    debug_assert!(n < 16);
    match n {
        0..=9 => b'0' + n,
        _ => b'a' + (n - 10),
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

// Trim ASCII whitespace and strip any optional 0x/0X prefix.
fn trim_hex(s: &str) -> &str {
    let s = s.trim();
    s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s)
}
