use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
    },
    stf::Tag,
};

pub struct Linebreak {}

impl Linebreak {
    pub fn new(tag: &Tag) -> Self {
        assert!(matches!(tag, Tag::Linebreak));

        Self {}
    }
}

impl Component<HtmlContext, HtmlInstruction> for Linebreak {
    fn generate(&mut self, _: &mut HtmlContext) -> Option<Vec<HtmlInstruction>> {
        Some(vec![HtmlInstruction::Line(Line { width: 0, data: String::from("<br>") })])
    }
}
