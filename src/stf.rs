use crate::util;

/// This type defines tags for the Style Tag Format. The Style Tag Format is a
/// very simple file format to add minimal styling to text.
///
/// Style Tag Format:
/// - Tags specify the following lines' (segment) style until the next tag => at
///   least one tag must be specified => the first line in the file must be a
///   tag
/// - Tags have the syntax >tag< using inward pointing angled brackets with the
///   tag name inbetween and must be the only text on a line.
/// - Tags may impose further syntax rules, those are only valid in the segment
///   following the tag.
pub enum Tag {
    /// A cover page containing a title, author, date and free form notes.
    Cover {
        // Single line.
        title: String,
        // Single line.
        author: String,
        // Single line.
        date: String,
        // Multiple lines:
        // - Single newlines should be ignored
        // - Double newlines should be interpreted as a single newline
        // - Any other configuration of consecutive newlines is undefined
        notes: String,
    },
    /// Configures the contents of page headers.
    HeaderConfig {
        // Single line.
        left: String,
        // Single line.
        right: String,
    },
    /// A generated Table of Contents using Headings to populate it.
    TableOfContents,
    /// Linebreak marker.
    Linebreak,
    /// Page break marker.
    Pagebreak,
    /// A heading like e.g. chapter title.
    Heading {
        // Multiple lines:
        // - Single newlines should be ignored
        // - Double newlines should be interpreted as a single newline
        // - Any other configuration of consecutive newlines is undefined
        content: String,
    },
    /// Plain text.
    // Multiple lines:
    // - Single newlines should be ignored
    // - Double newlines should be interpreted as a single newline
    // - Any other configuration of consecutive newlines is undefined
    Text(String),
    /// Code.
    // Multiple lines.
    Code(String),
    /// A hyperlink.
    Link {
        // Single line.
        url: String,
        // Single line.
        abbrev: String,
        // Multiple lines:
        // - Single newlines should be ignored
        // - Double newlines should be interpreted as a single newline
        // - Any other configuration of consecutive newlines is undefined
        content: String,
    },
}

pub fn parse(text: &str) -> impl Iterator<Item = Tag> + Clone {
    let mut remainder = text.trim_start();

    std::iter::from_fn(move || {
        loop {
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

            return match tag {
                "cover" => Some(Tag::Cover {
                    title: content.next().unwrap_or("").to_string(),
                    author: content.next().unwrap_or("").to_string(),
                    date: content.next().unwrap_or("").to_string(),
                    notes: util::collapse(content.remainder().unwrap_or("").trim_end()),
                }),
                "headerconfig" => Some(Tag::HeaderConfig {
                    left: content.next().unwrap_or("").to_string(),
                    right: content.next().unwrap_or("").to_string(),
                }),
                "toc" => Some(Tag::TableOfContents),
                "linebreak" => Some(Tag::Linebreak),
                "pagebreak" => Some(Tag::Pagebreak),
                "heading" => {
                    Some(Tag::Heading { content: util::collapse(content.remainder().unwrap_or("").trim_end()) })
                }
                "text" => Some(Tag::Text(util::collapse(content.remainder().unwrap_or("").trim_end()))),
                "code" => Some(Tag::Code(content.remainder().unwrap_or("").trim_end().to_string())),
                "link" => Some(Tag::Link {
                    url: content.next().unwrap_or("").to_string(),
                    abbrev: content.next().unwrap_or("").to_string(),
                    content: util::collapse(content.remainder().unwrap_or("").trim_end()),
                }),
                // Skip unimplemented tags.
                _ => continue,
            };
        }
    })
}
