use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
    },
    stf::Tag,
};

pub struct Pagebreak {}

impl Pagebreak {
    pub fn new(tag: &Tag) -> Self {
        assert!(matches!(tag, Tag::Pagebreak));

        Self {}
    }
}

impl Component<TxtContext, TxtInstruction> for Pagebreak {
    fn generate(&mut self, _: &mut TxtContext) -> Option<Vec<TxtInstruction>> { Some(vec![TxtInstruction::Pagebreak]) }
}
