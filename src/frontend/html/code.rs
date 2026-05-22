use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
    util,
};

pub struct Code {
    content: String,
}

impl Code {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Code(_)));
        let Tag::Code(content) = tag else { unreachable!() };

        Self { content }
    }
}

impl Component<HtmlContext, HtmlInstruction> for Code {
    fn generate(&mut self, ctx: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        let mut lines: Vec<Line> = util::wrap_code(&self.content, ctx.width)
            .map(|line| Line {
                width: line.graphemes(true).count(),
                data: format!("<code>{}</code><br>", util::escape(line)),
            })
            .collect();

        let first = lines.first_mut().unwrap();
        first.data = format!("<pre class=\"box code\">{}", first.data);

        let last = lines.last_mut().unwrap();
        last.data = format!("{}</pre>", last.data);

        Some(vec![HtmlInstruction::Block(lines), HtmlInstruction::Padding])
    }
}
