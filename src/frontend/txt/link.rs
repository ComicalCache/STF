use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
    },
    stf::Tag,
    util,
};

pub struct Link {
    url: String,
    content: String,
}

impl Link {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Link { .. }));
        let Tag::Link { url, content, .. } = tag else { unreachable!() };

        Self { url, content }
    }
}

impl Component<TxtContext, TxtInstruction> for Link {
    fn generate(&mut self, ctx: &mut TxtContext) -> Option<Vec<TxtInstruction>> {
        Some(vec![TxtInstruction::Paragraph(
            util::wrap_paragraph(&format!("{}: {}", self.content, self.url), ctx.width).map(String::from).collect(),
        )])
    }
}
