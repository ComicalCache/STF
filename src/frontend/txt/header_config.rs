use crate::{
    frontend::{
        components::Component,
        txt::{Header, TxtContext, TxtInstruction},
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

impl Component<TxtContext, TxtInstruction> for HeaderConfig {
    fn configure(&mut self, ctx: &mut TxtContext) {
        let left: Vec<String> = util::wrap_paragraph(&self.left, ctx.width).map(String::from).collect();
        let right: Vec<String> = util::wrap_paragraph(&self.right, ctx.width).map(String::from).collect();

        ctx.header = Some(Header { left, right });
    }
}
