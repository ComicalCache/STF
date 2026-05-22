use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
    util,
};

pub struct Heading {
    content: String,
}

impl Heading {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Heading { .. }));
        let Tag::Heading { content } = tag else { unreachable!() };

        Self { content }
    }
}

impl Component<HtmlContext, HtmlInstruction> for Heading {
    fn generate(&mut self, ctx: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        let lines: Vec<Line> = util::wrap_paragraph(&self.content, ctx.width)
            .map(|line| Line {
                width: line.graphemes(true).count(),
                data: format!("<span class=\"box heading align-center\">{}</span><br>", util::escape(line)),
            })
            .collect();

        Some(vec![
            HtmlInstruction::RegisterToC(self.content.clone()),
            HtmlInstruction::Block(lines),
            HtmlInstruction::Padding,
        ])
    }
}
