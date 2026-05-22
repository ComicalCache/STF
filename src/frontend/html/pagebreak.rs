use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction},
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

impl Component<HtmlContext, HtmlInstruction> for Pagebreak {
    fn generate(&mut self, _: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        Some(vec![HtmlInstruction::Pagebreak])
    }
}
