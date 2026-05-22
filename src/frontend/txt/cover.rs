use crate::{
    frontend::{
        components::Component,
        txt::{Page, TxtContext, TxtInstruction},
    },
    stf::Tag,
    util,
};

pub struct Cover {
    title: String,
    author: String,
    date: String,
    notes: String,
}

impl Cover {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::Cover { .. }));
        let Tag::Cover { title, author, date, notes } = tag else { unreachable!() };

        Self { title, author, date, notes }
    }
}

impl Component<TxtContext, TxtInstruction> for Cover {
    fn modify(&mut self, ctx: &mut TxtContext) {
        // Leave 25% top padding.
        let top_padding = (ctx.max_lines as f32 * 0.25) as usize;
        let mut cover = Page::new(ctx.width, ctx.max_lines, 0, top_padding, 0, 0, 7);

        for line in util::wrap_paragraph(&self.title, ctx.width) {
            cover.push_body(format!("{line:^width$}", width = ctx.width));
        }
        // One line padding between title and author
        cover.push_body(String::new());
        for line in util::wrap_paragraph(&self.author, ctx.width) {
            cover.push_body(format!("{line:^width$}", width = ctx.width));
        }
        for line in util::wrap_paragraph(&self.date, ctx.width) {
            cover.push_body(format!("{line:^width$}", width = ctx.width));
        }

        // Add the notes to the footnotes.
        for line in util::wrap_paragraph(&self.notes, ctx.width) {
            cover.push_footnote(format!("{line:^width$}", width = ctx.width));
        }

        ctx.doc.cover = Some(cover);
    }
}
