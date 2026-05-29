//! Shared helpers used across the parser, layout, and renderer.

/// Return true for the five HTML collapsible whitespace characters.
pub(crate) fn is_html_collapsible_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

/// Trim only HTML collapsible whitespace from the end of a string.
pub(crate) fn trim_html_collapsible_whitespace_end(s: &str) -> String {
    let mut end = s.len();

    for (idx, c) in s.char_indices().rev() {
        if is_html_collapsible_whitespace(c) {
            end = idx;
        } else {
            break;
        }
    }

    s[..end].to_string()
}

/// Return true when a string contains a Unicode non-breaking space.
pub(crate) fn contains_nbsp(s: &str) -> bool {
    s.contains('\u{00A0}')
}

/// Decode a standard Base64 string without pulling in an extra dependency.
pub(crate) fn decode_base64(input: &str) -> Option<Vec<u8>> {
    fn table(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut chunks = bytes.chunks_exact(4);

    for chunk in &mut chunks {
        let a = table(chunk[0])?;
        let b = table(chunk[1])?;
        result.push((a << 2) | (b >> 4));

        if chunk[2] != b'=' {
            let c = table(chunk[2])?;
            result.push((b << 4) | (c >> 2));

            if chunk[3] != b'=' {
                let d = table(chunk[3])?;
                result.push((c << 6) | d);
            }
        }
    }

    match chunks.remainder() {
        [] | [_] => {}
        [a, b] => {
            let a = table(*a)?;
            let b = table(*b)?;
            result.push((a << 2) | (b >> 4));
        }
        [a, b, c] => {
            let a = table(*a)?;
            let b = table(*b)?;
            result.push((a << 2) | (b >> 4));
            if *c != b'=' {
                let c = table(*c)?;
                result.push((b << 4) | (c >> 2));
            }
        }
        _ => return None,
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::decode_base64;

    #[test]
    fn decode_base64_basic() {
        assert_eq!(
            decode_base64("SGVsbG8=").as_deref(),
            Some(b"Hello".as_ref())
        );
    }

    #[test]
    fn decode_base64_with_whitespace() {
        assert_eq!(
            decode_base64("SGVs\nbG8=").as_deref(),
            Some(b"Hello".as_ref())
        );
    }

    #[test]
    fn decode_base64_ignores_single_trailing_byte() {
        assert_eq!(
            decode_base64("SGVsbG8=A").as_deref(),
            Some(b"Hello".as_ref())
        );
    }

    #[test]
    fn decode_base64_empty_string() {
        assert_eq!(decode_base64("").as_deref(), Some(b"".as_ref()));
    }

    #[test]
    fn decode_base64_no_padding_two_chars() {
        // "YQ" is "a" without padding (base64 of b"a" is "YQ==")
        assert_eq!(decode_base64("YQ").as_deref(), Some(b"a".as_ref()));
    }

    #[test]
    fn decode_base64_no_padding_three_chars() {
        // "YWI" is "ab" without padding (base64 of b"ab" is "YWI=")
        assert_eq!(decode_base64("YWI").as_deref(), Some(b"ab".as_ref()));
    }

    #[test]
    fn decode_base64_invalid_character_returns_none() {
        assert!(decode_base64("SG!s").is_none());
    }

    #[test]
    fn decode_base64_another_invalid_character_returns_none() {
        // '@' is not a valid base64 character
        assert!(decode_base64("SGVs@G8=").is_none());
    }

    #[test]
    fn decode_base64_longer_multi_block_string() {
        // base64 of b"The quick brown fox"
        assert_eq!(
            decode_base64("VGhlIHF1aWNrIGJyb3duIGZveA==").as_deref(),
            Some(b"The quick brown fox".as_ref())
        );
    }

    #[test]
    fn decode_base64_longer_string_no_padding() {
        // base64 of b"ironpress" is "aXJvbnByZXNz" (no padding needed — 9 bytes → 12 chars)
        assert_eq!(
            decode_base64("aXJvbnByZXNz").as_deref(),
            Some(b"ironpress".as_ref())
        );
    }
}
