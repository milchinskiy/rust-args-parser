/// Compute visible length (strip minimal ANSI we add) so alignment stays correct.
#[must_use]
pub fn strip_ansi_len(s: &str) -> usize {
    let mut n = 0usize;
    let mut skip = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B {
            skip = true;
            i += 1;
            continue;
        }
        if skip {
            if bytes[i] == b'm' {
                skip = false;
            }
            i += 1;
            continue;
        }
        n += 1;
        i += 1;
    }
    n
}

/// Heuristic: token that looks like a number (accepts leading '-', digits, optional dot/exponent).
#[must_use]
pub fn looks_like_number_token(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[0] == b'-' {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
    let mut has_digit = false;
    let mut dot = false;
    let mut exp = false;
    while i < bytes.len() {
        match bytes[i] {
            b'0'..=b'9' => {
                has_digit = true;
            }
            b'.' if !dot && !exp => {
                dot = true;
            }
            b'e' | b'E' if has_digit && !exp => {
                exp = true;
                has_digit = false;
                if i + 1 < bytes.len() && (bytes[i + 1] == b'+' || bytes[i + 1] == b'-') {
                    i += 1;
                }
            }
            _ => return false,
        }
        i += 1;
    }
    has_digit
}
