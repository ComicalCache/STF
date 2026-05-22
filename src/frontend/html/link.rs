use std::fmt::Write;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
    util,
};

pub struct Link {
    url: String,
    abbrev: String,
    content: String,
}

impl Link {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Link { .. }));
        let Tag::Link { url, abbrev, content } = tag else { unreachable!() };

        Self { url, abbrev, content }
    }
}

impl Component<HtmlContext, HtmlInstruction> for Link {
    fn generate(&mut self, ctx: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        let text = format!("{} ({})", self.content, self.abbrev);

        // Plus two for the space and opening bracket.
        let abbrev_start = self.content.len() + 2;
        let abbrev_end = abbrev_start + self.abbrev.len();

        let lines: Vec<Line> = util::wrap_paragraph(&text, ctx.width)
            .map(|line| {
                let width = line.graphemes(true).count();
                let offset = line.as_ptr() as usize - text.as_ptr() as usize;

                let start = (abbrev_start.saturating_sub(offset)).min(line.len());
                let stop = (abbrev_end.saturating_sub(offset)).min(line.len());

                let before = &line[..start];
                let inside = &line[start..stop];
                let after = &line[stop..];

                let mut data = String::from("<span>");
                if !before.is_empty() {
                    data.push_str(&util::escape(before));
                }
                if !inside.is_empty() {
                    let _ = write!(data, "<a href=\"{}\">{}</a>", self.url, util::escape(inside));
                }
                if !after.is_empty() {
                    data.push_str(&util::escape(after));
                }
                data.push_str("</span><br>");

                Line { width, data }
            })
            .collect();

        Some(vec![HtmlInstruction::Paragraph(lines)])
    }
}
