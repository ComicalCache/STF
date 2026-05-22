use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
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

        Code { content }
    }
}

impl Component<TxtContext, TxtInstruction> for Code {
    fn generate(&mut self, ctx: &mut TxtContext) -> Option<Vec<TxtInstruction>> {
        Some(vec![TxtInstruction::Block(util::wrap_code(&self.content, ctx.width).map(String::from).collect())])
    }
}
