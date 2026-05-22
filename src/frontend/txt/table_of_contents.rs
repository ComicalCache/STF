use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        txt::{TxtContext, TxtInstruction},
    },
    stf::Tag,
    util,
};

pub struct TableOfContents {}

impl TableOfContents {
    pub fn new(tag: Tag) -> Self {
        assert!(matches!(tag, Tag::TableOfContents));

        TableOfContents {}
    }
}

impl Component<TxtContext, TxtInstruction> for TableOfContents {
    fn modify(&mut self, ctx: &mut TxtContext) {
        if ctx.headings.is_empty() {
            return;
        }

        let mut toc = vec![ctx.new_page()];
        toc[0].push_body(format!("{:^width$}", "Table Of Contents", width = ctx.width));
        toc[0].push_body(String::new());

        let max_page_num_width = ctx.headings.last().unwrap().0.checked_ilog10().unwrap_or(0) as usize + 1;
        for (page_nr, title) in &ctx.headings {
            // Minus three for enough space for at least one dot.
            let mut entry: Vec<_> =
                util::wrap_paragraph(title, ctx.width - max_page_num_width - 3).map(str::to_string).collect();

            let toc_page_nr = toc.len();
            let page = toc.last_mut().unwrap();
            if !page.fits(entry.len()) {
                let page_nr = format!("[Page {}]", util::to_roman(toc_page_nr));
                page.push_footer(format!("{page_nr:>width$}", width = ctx.width));

                toc.push(ctx.new_page());
            }
            let page = toc.last_mut().unwrap();

            // Save last line as special case.
            let last_line = entry.pop().unwrap();

            // Push all lines of the title.
            for line in entry {
                page.push_body(line);
            }

            // Pad last line with trailing dots and page number.
            let last_line_graphemes = last_line.graphemes(true).count();
            page.push_body(format!(
                "{last_line} {:.>width$} {page_nr:>max_page_num_width$}",
                "",
                width = ctx.width - last_line_graphemes - max_page_num_width - 2
            ));
        }

        // Add page number to last page.
        let page_nr = format!("[Page {}]", util::to_roman(toc.len()));
        toc.last_mut().unwrap().push_footer(format!("{page_nr:>width$}", width = ctx.width));

        // Merge ToC and Document.
        let mut pages = std::mem::take(&mut ctx.doc.pages);
        toc.append(&mut pages);
        ctx.doc.pages = toc;
    }
}
