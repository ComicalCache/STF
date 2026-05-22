use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
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

        Self { content }
    }
}

impl Component<TxtContext, TxtInstruction> for Text {
    fn generate(&mut self, ctx: &mut TxtContext) -> Option<Vec<TxtInstruction>> {
        Some(vec![TxtInstruction::Paragraph(
            util::wrap_paragraph(&self.content, ctx.width).map(String::from).collect(),
        )])
    }
}
