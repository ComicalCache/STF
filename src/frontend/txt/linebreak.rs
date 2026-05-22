use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
    },
    stf::Tag,
};

pub struct Linebreak {}

impl Linebreak {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Linebreak));

        Linebreak {}
    }
}

impl Component<TxtContext, TxtInstruction> for Linebreak {
    fn generate(&mut self, _: &mut TxtContext) -> Option<Vec<TxtInstruction>> {
        Some(vec![TxtInstruction::Line(String::new())])
    }
}
