// This file implements a parser for the Style Tag Format. The Style Tag Format is a very simple file format to add
// minimal styling to text.
//
// Style Tag Format:
// - Tags specify the following lines' (segment) style until the next tag
//      => at least one tag must be specified
//      => the first line in the file must be a tag
// - Tags have the syntax >TAG< using inward pointing angled brackets with the tag name inbetween and must be the only
//      text on a line.
// - Tags may impose further syntax rules, those are only valid in the segment following the tag.

use crate::util;

/// This parser supported tags. If not stated otherwise, newlines are ignored! Tags are spelt lowercase!
pub enum Tag<'s> {
    Cover {
        title: &'s str,
        author: &'s str,
        date: &'s str,
        /// - Single newlines should be ignored
        /// - Double newlines should be interpreted as a single newline
        /// - Any other configuration of consecutive newlines is undefined
        notes: String,
    },
    Header {
        date: &'s str,
        title: &'s str,
    },
    TableOfContents,
    Linebreak,
    Pagebreak,
    Heading {
        content: String,
    },
    Text {
        /// - Single newlines should be ignored
        /// - Double newlines should be interpreted as a single newline
        /// - Any other configuration of consecutive newlines is undefined
        content: String,
    },
    Code {
        content: &'s str,
    },
}

/// Zero allocation Style Tag Format parser.
///
/// The `Clone` trait is needed to duplicate the iterator. This is cheap since &str is cheap to clone.
pub fn parse(text: &str) -> impl Iterator<Item = Tag<'_>> + Clone + '_ {
    let mut remainder = text.trim_start();

    std::iter::from_fn(move || {
        if remainder.is_empty() {
            return None;
        }

        // This must be a tag.
        let tag_end = remainder.find('\n').unwrap_or(remainder.len());
        let mut tag = &remainder[..tag_end];
        tag = tag.strip_prefix('>')?.strip_suffix('<')?;

        let mut content_start = tag_end;
        if content_start < remainder.len() {
            content_start += 1; // Skip newline.
        }

        let mut next_tag_start = remainder.len();
        let mut offset = content_start;
        while offset < remainder.len() {
            let tail = &remainder[offset..];
            let next_line = tail.find('\n').unwrap_or(tail.len());

            let line = &tail[..next_line];

            // Found next tag.
            if line.starts_with('>') && line.ends_with('<') {
                next_tag_start = offset;
                break;
            }

            offset += next_line;
            if offset < remainder.len() {
                offset += 1; // Skip newline.
            }
        }

        let content = &remainder[content_start..next_tag_start];
        remainder = &remainder[next_tag_start..];
        let mut content = content.lines();
        match tag {
            "cover" => Some(Tag::Cover {
                title: content.next().unwrap_or(""),
                author: content.next().unwrap_or(""),
                date: content.next().unwrap_or(""),
                notes: util::collapse(content.remainder().unwrap_or("")),
            }),
            "header" => Some(Tag::Header { date: content.next().unwrap_or(""), title: content.next().unwrap_or("") }),
            "toc" => Some(Tag::TableOfContents),
            "linebreak" => Some(Tag::Linebreak),
            "pagebreak" => Some(Tag::Pagebreak),
            "heading" => Some(Tag::Heading { content: util::collapse(content.remainder().unwrap_or("")) }),
            "text" => Some(Tag::Text { content: util::collapse(content.remainder().unwrap_or("")) }),
            "code" => Some(Tag::Code { content: content.remainder().unwrap_or("") }),
            _ => None,
        }
    })
}
