use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{Header, HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
    util,
};

pub struct HeaderConfig {
    left: String,
    right: String,
}

impl HeaderConfig {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::HeaderConfig { .. }));
        let Tag::HeaderConfig { left, right } = tag else { unreachable!() };

        Self { left, right }
    }
}

impl Component<HtmlContext, HtmlInstruction> for HeaderConfig {
    fn configure(&mut self, ctx: &mut HtmlContext) {
        let left: Vec<Line> = util::wrap_paragraph(&self.left, ctx.width)
            .map(|line| Line { width: line.graphemes(true).count(), data: format!("<p>{}</p>", util::escape(line)) })
            .collect();

        let right: Vec<Line> = util::wrap_paragraph(&self.right, ctx.width)
            .map(|line| Line {
                width: line.graphemes(true).count(),
                data: format!("<p class=\"align-right italic\">{}</p>", util::escape(line)),
            })
            .collect();

        ctx.header = Some(Header { left, right });
    }
}
