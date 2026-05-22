use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
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

impl Component<TxtContext, TxtInstruction> for Heading {
    fn generate(&mut self, ctx: &mut TxtContext) -> Option<Vec<TxtInstruction>> {
        Some(vec![
            TxtInstruction::Block(
                util::wrap_paragraph(&self.content, ctx.width)
                    .map(|line| format!("{line:^width$}", width = ctx.width))
                    .collect(),
            ),
            TxtInstruction::RegisterToC(self.content.clone()),
            TxtInstruction::Padding,
        ])
    }
}
