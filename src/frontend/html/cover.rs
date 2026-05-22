use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line, Page},
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

impl Component<HtmlContext, HtmlInstruction> for Cover {
    fn modify(&mut self, ctx: &mut HtmlContext) {
        // Leave 25% top padding.
        let top_padding = (ctx.max_lines as f32 * 0.25) as usize;
        let mut cover = Page::new(ctx.width, ctx.max_lines, 0, top_padding, 0, 0, 7);

        for line in util::wrap_paragraph(&self.title, ctx.width) {
            let width = line.graphemes(true).count();
            let data = format!("<p class=\"align-center heading\">{}</p>", util::escape(line));
            cover.push_body(Line { width, data });
        }
        // One line padding between title and author
        cover.push_body(Line { width: 0, data: String::from("<br>") });

        for line in util::wrap_paragraph(&self.author, ctx.width) {
            let width = line.graphemes(true).count();
            let data = format!("<p class=\"align-center italic\">{}</p>", util::escape(line));
            cover.push_body(Line { width, data });
        }
        for line in util::wrap_paragraph(&self.date, ctx.width) {
            let width = line.graphemes(true).count();
            let data = format!("<p class=\"align-center\">{}</p>", util::escape(line));
            cover.push_body(Line { width, data });
        }

        // Add the notes to the footnotes.
        for line in util::wrap_paragraph(&self.notes, ctx.width) {
            let width = line.graphemes(true).count();
            let data = format!("<p class=\"align-center\">{}</p>", util::escape(line));
            cover.push_footnote(Line { width, data });
        }

        ctx.doc.cover = Some(cover);
    }
}
