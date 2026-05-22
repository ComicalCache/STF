use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        html::{HtmlContext, HtmlInstruction, Line},
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

impl Component<HtmlContext, HtmlInstruction> for TableOfContents {
    fn modify(&mut self, ctx: &mut HtmlContext) {
        if ctx.headings.is_empty() {
            return;
        }

        let mut toc = vec![ctx.new_page()];

        let title = "Table Of Contents";
        toc[0].push_body(Line {
            width: title.graphemes(true).count(),
            data: format!("<span class=\"box heading align-center\">{title}</span><br>"),
        });
        toc[0].push_body(Line { width: 0, data: String::from("<br>") });

        let max_page_num_width = ctx.headings.last().unwrap().0.checked_ilog10().unwrap_or(0) as usize + 1;
        for (page_nr, title) in &ctx.headings {
            // Minus three for enough space for at least one dot/line.
            let mut entry: Vec<_> =
                util::wrap_paragraph(title, ctx.width - max_page_num_width - 3).map(str::to_string).collect();

            let toc_page_nr = toc.len();
            let page = toc.last_mut().unwrap();
            if !page.fits(entry.len()) {
                let page_nr = format!("[Page {}]", util::to_roman(toc_page_nr));
                page.push_footer(Line {
                    width: page_nr.graphemes(true).count(),
                    data: format!("<p class=\"align-right\">{page_nr}</p>"),
                });

                toc.push(ctx.new_page());
            }
            let page = toc.last_mut().unwrap();

            // Save last line as special case.
            let last_line = entry.pop().unwrap();

            // Push all lines of the title.
            for line in entry {
                let width = line.graphemes(true).count();
                page.push_body(Line { width, data: format!("<span class=\"box\">{}</span><br>", util::escape(&line)) });
            }

            // Pad last line with the HTML flexbox row and page number.
            let width = last_line.graphemes(true).count() + 3 + max_page_num_width;
            page.push_body(Line {
                width,
                data: format!(
                    "<div class=\"box\"><div class=\"toc-row\"><span>{}</span><span class=\"toc-line\"></span>\
                    <span>{page_nr:>max_page_num_width$}</span></div></div>",
                    util::escape(&last_line),
                    max_page_num_width = max_page_num_width
                ),
            });
        }

        // Add page number to last page.
        let page_nr = format!("[Page {}]", util::to_roman(toc.len()));
        toc.last_mut().unwrap().push_footer(Line {
            width: page_nr.graphemes(true).count(),
            data: format!("<p class=\"align-right\">{page_nr}</p>"),
        });

        // Merge ToC and Document.
        let mut pages = std::mem::take(&mut ctx.doc.pages);
        toc.append(&mut pages);
        ctx.doc.pages = toc;
    }
}
