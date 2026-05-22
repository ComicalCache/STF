use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
    util,
};

pub struct Text {
    content: String,
}

impl Text {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Text(_)));
        let Tag::Text(content) = tag else { unreachable!() };

        Text { content }
    }
}

impl Component<HtmlContext, HtmlInstruction> for Text {
    fn generate(&mut self, ctx: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        let lines: Vec<Line> = util::wrap_paragraph(&self.content, ctx.width)
            .map(|line| Line {
                width: line.graphemes(true).count(),
                data: format!("<span>{}</span><br>", util::escape(line)),
            })
            .collect();

        Some(vec![HtmlInstruction::Paragraph(lines)])
    }
}
