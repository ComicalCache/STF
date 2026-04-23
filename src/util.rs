use unicode_segmentation::UnicodeSegmentation;

/// Collapses double newlines `\n\n` into a single `\n` and single newlines `\n` into a whitespace ` `.
pub fn collapse(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    let mut count = 0;
    for ch in text.chars() {
        if ch == '\n' {
            count += 1;
        } else {
            match count {
                0 => {}
                1 => result.push(' '),
                _ => result.push('\n'),
            }

            count = 0;
            result.push(ch);
        }
    }

    // Trailing newlines.
    match count {
        0 => {}
        1 => result.push(' '),
        _ => result.push('\n'),
    }

    result
}

/// Zero allocation paragraph word wrapping.
///
/// Takes in `text` and returns an iterator over the wrapped lines where each line has less than `width` graphemes.
///
/// Leading and trailing whitespace of wrapped lines is trimmed! Words that exceed `width` graphemes are greedily cut at
/// the `width`'th grapheme, without regard for hyphenation! Correctly typed punctuation will never start a line!
///
/// The `Clone` trait is needed to duplicate the iterator. This is cheap since &str is cheap to clone.
pub fn wrap_paragraph(text: &str, width: usize) -> impl Iterator<Item = &str> + Clone + '_ { wrap(text, width, true) }

/// Zero allocation code word wrapping.
///
/// Takes in `code` and returns an iterator over the wrapped lines where each line has less than `width` graphemes.
///
/// This function is identical to `wrap_paragraph` but doesn't trim leading whitespace for indentation!
pub fn wrap_code(code: &str, width: usize) -> impl Iterator<Item = &str> + Clone + '_ { wrap(code, width, false) }

fn wrap(text: &str, width: usize, skip_leading_whitespace: bool) -> impl Iterator<Item = &str> + Clone + '_ {
    assert!(width > 0);

    let mut remainder = text;
    std::iter::from_fn(move || {
        if skip_leading_whitespace {
            remainder = remainder.trim_start();
        }

        if remainder.is_empty() {
            return None;
        }

        let mut count = 0;
        let mut end = 0;

        // Use while let to not take ownership of the iterator.
        let mut tokens = remainder.split_word_bounds().peekable();
        while let Some(token) = tokens.next() {
            // Break line.
            if let Some(idx) = token.find('\n') {
                // Plus one to consume the newline character.
                let (res, new_remainder) = remainder.split_at(end + idx + 1);
                remainder = new_remainder;

                // Skip trailing whitespace.
                return Some(res.trim_end());
            }

            let mut chunk_count = token.graphemes(true).count();
            let mut bytes = token.len();

            // Group trailing punctuation and symbols with the current token.
            while let Some(&next_token) = tokens.peek() {
                let whitespace = next_token.chars().all(char::is_whitespace);
                let alphanumeric = next_token.chars().any(char::is_alphanumeric);

                if whitespace || alphanumeric {
                    break;
                }

                chunk_count += next_token.graphemes(true).count();
                bytes += next_token.len();
                tokens.next();
            }

            // Token exceeds the line limit.
            if count + chunk_count > width {
                if count == 0 {
                    // Word is larger than max_line_len => cut at the limit.
                    let byte = remainder.grapheme_indices(true).nth(width).map_or(remainder.len(), |(idx, _)| idx);

                    let (res, new_remainder) = remainder.split_at(byte);
                    remainder = new_remainder;

                    // Skip trailing whitespace.
                    return Some(res.trim_end());
                }

                // Continue to the next line.
                break;
            }

            count += chunk_count;
            end += bytes;
        }

        let (res, new_remainder) = remainder.split_at(end);
        remainder = new_remainder;

        // Skip trailing whitespace.
        Some(res.trim_end())
    })
}
