mod code;
mod cover;
mod header_config;
mod heading;
mod linebreak;
mod link;
mod page;
mod pagebreak;
mod table_of_contents;
mod text;

use std::fmt::Write;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    frontend::{
        components::Component,
        document::Document,
        html::{
            code::Code,
            cover::Cover,
            header_config::HeaderConfig,
            heading::Heading,
            linebreak::Linebreak,
            link::Link,
            page::{Line, Page},
            pagebreak::Pagebreak,
            table_of_contents::TableOfContents,
            text::Text,
        },
    },
    stf::Tag,
    util,
};

struct Header {
    left: Vec<Line>,
    right: Vec<Line>,
}

pub struct HtmlContext {
    /// Width in graphemes.
    width: usize,
    /// Maximum number of lines per page (including header and footer).
    max_lines: usize,

    header: Option<Header>,

    /// Headings on page with title.
    headings: Vec<(usize, String)>,

    doc: Document<Page>,
}

impl HtmlContext {
    const fn new(width: usize, max_lines: usize) -> Self {
        Self { width, max_lines, header: None, headings: Vec::new(), doc: Document::new() }
    }

    fn new_page(&self) -> Page {
        let mut page = Page::new(
            self.width,
            self.max_lines,
            self.header.as_ref().map_or(0, |header| header.left.len() + header.right.len()),
            1,
            1,
            1, // The line number.
            1,
        );

        if let Some(header) = &self.header {
            for line in &header.left {
                page.push_header(line.clone());
            }

            for line in &header.right {
                page.push_header(line.clone());
            }
        }

        page
    }

    fn last_page(&mut self) -> &mut Page {
        if self.doc.pages.is_empty() {
            self.doc.pages.push(self.new_page());
        }

        self.doc.pages.last_mut().unwrap()
    }

    fn break_page(&mut self) -> &mut Page {
        self.doc.pages.push(self.new_page());
        self.doc.pages.last_mut().unwrap()
    }
}

pub enum HtmlInstruction {
    /// A single line.
    Line(Line),
    /// A block that must not be broken by a pagebreak.
    Block(Vec<Line>),
    /// A paragraph that may not have a lone line on the previous or following
    /// page.
    Paragraph(Vec<Line>),
    /// Creates a new page.
    Pagebreak,
    /// Adds a line of padding.
    Padding,
    /// Registers a Table of Contents entry at the current page.
    RegisterToC(String),
}

type HtmlComponent = Box<dyn Component<HtmlContext, HtmlInstruction>>;

pub struct Html {
    components: Vec<HtmlComponent>,
    ctx: HtmlContext,
}

impl Html {
    #[allow(clippy::too_many_lines)]
    pub fn generate(title: &str, tags: impl Iterator<Item = Tag>, width: usize, max_lines: usize) -> String {
        let mut html = Self {
            components: tags
                .map(|tag| match tag {
                    Tag::Cover { .. } => Box::new(Cover::new(tag)) as HtmlComponent,
                    Tag::HeaderConfig { .. } => Box::new(HeaderConfig::new(tag)) as HtmlComponent,
                    Tag::TableOfContents => Box::new(TableOfContents::new(&tag)) as HtmlComponent,
                    Tag::Linebreak => Box::new(Linebreak::new(&tag)) as HtmlComponent,
                    Tag::Pagebreak => Box::new(Pagebreak::new(&tag)) as HtmlComponent,
                    Tag::Heading { .. } => Box::new(Heading::new(tag)) as HtmlComponent,
                    Tag::Text(_) => Box::new(Text::new(tag)) as HtmlComponent,
                    Tag::Code(_) => Box::new(Code::new(tag)) as HtmlComponent,
                    Tag::Link { .. } => Box::new(Link::new(tag)) as HtmlComponent,
                })
                .collect(),
            ctx: HtmlContext::new(width, max_lines),
        };

        // First pass for Components that need setup.
        for component in &mut html.components {
            component.configure(&mut html.ctx);
        }

        // Second pass for Components that can write their contents.
        let mut components = std::mem::take(&mut html.components);
        for component in &mut components {
            let Some(instructions) = component.generate(&mut html.ctx) else {
                continue;
            };

            for instruction in instructions {
                match instruction {
                    HtmlInstruction::Line(data) => {
                        let page = html.ctx.last_page();

                        if page.fits(1) {
                            page.push_body(data);
                        } else {
                            html.ctx.break_page().push_body(data);
                        }
                    }
                    HtmlInstruction::Block(lines) => {
                        assert!(lines.len() <= max_lines);

                        let page = html.ctx.last_page();

                        if page.fits(lines.len()) {
                            for line in lines {
                                page.push_body(line);
                            }
                        } else {
                            let page = html.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        }
                    }
                    HtmlInstruction::Paragraph(mut lines) => {
                        let page = html.ctx.last_page();

                        if page.fits(lines.len()) {
                            // All lines of the paragraph fit on the current page.

                            for line in lines {
                                page.push_body(line);
                            }
                        } else if page.fits(lines.len().saturating_sub(2)) {
                            // Avoid having a single trailing line cause a page break, break with two
                            // trailing lines instead.

                            for line in lines.drain(0..lines.len().saturating_sub(2)) {
                                page.push_body(line);
                            }
                            let page = html.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        } else if page.fits(1) {
                            // Avoid having a single leading line cause a page
                            // break, break immediately.

                            let page = html.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        }
                    }
                    HtmlInstruction::Pagebreak => {
                        html.ctx.break_page();
                    }
                    HtmlInstruction::Padding => {
                        let page = html.ctx.last_page();

                        if !page.body().is_empty() {
                            if page.fits(1) {
                                page.push_body(Line { width: 0, data: String::from("<br>") });
                            } else {
                                html.ctx.break_page();
                            }
                        }
                    }
                    HtmlInstruction::RegisterToC(title) => html.ctx.headings.push((html.ctx.doc.pages.len(), title)),
                }
            }
        }
        html.components = components;

        // Add page numbers generated pages.
        let width = html.ctx.width;
        for (idx, page) in html.ctx.doc.pages.iter_mut().enumerate() {
            let page_nr = format!("[Page {}]", idx + 1);
            let width = page_nr.graphemes(true).count();
            page.push_footer(Line { width, data: format!("<p class=\"align-right\">{page_nr}</p>") });
        }

        // Third pass for Components that need information added by other, already
        // lay-outed Components.
        for component in &mut html.components {
            component.modify(&mut html.ctx);
        }

        // Assemble the Document into a String.
        let assemble = |page: &Page| {
            let mut lines = Vec::with_capacity(page.max_lines);

            lines.extend(page.header().iter().cloned().map(|line| line.data));
            // Pad to the minimum header lines.
            for _ in page.header().len()..page.min_header {
                lines.push(String::from("<br>"));
            }
            for _ in 0..page.header_padding {
                lines.push(String::from("<br>"));
            }

            lines.extend(page.body().iter().cloned().map(|line| line.data));

            // Padding.
            for _ in 0..page.max_lines.saturating_sub(page.lines()) {
                lines.push(String::from("<br>"));
            }

            for _ in 0..page.footnotes_padding {
                lines.push(String::from("<br>"));
            }
            lines.extend(page.footnotes().iter().cloned().map(|line| line.data));

            for _ in 0..page.footer_padding {
                lines.push(String::from("<br>"));
            }
            // Pad to the minimum footer lines.
            for _ in page.footer().len()..page.min_footer {
                lines.push(String::from("<br>"));
            }
            lines.extend(page.footer().iter().cloned().map(|line| line.data));

            lines.join("")
        };

        let cover = html
            .ctx
            .doc
            .cover
            .map(|page| assemble(&page) + "<br><div class=\"pagebreak\"></div><br>")
            .unwrap_or_default();
        let pages = html
            .ctx
            .doc
            .pages
            .iter()
            .map(assemble)
            .collect::<Vec<String>>()
            .join("<br><div class=\"pagebreak\"></div><br>");
        let content = cover + &pages + "<br><div class=\"pagebreak\"></div><br>";

        let mut final_html = String::new();
        final_html.push_str("<!DOCTYPE html><html><head><meta charset=\"UTF-8\">");
        let _ = write!(
            final_html,
            "<title>{}</title><style>body{{font-family:monospace}}.main{{max-width:{width}ch;width:100%;margin:0 auto}}.align-center{{text-align:center}}.align-right{{text-align:right}}.pagebreak{{height:0;border-top:1px solid #ccc;width:100%}}.heading{{font-size:1em;font-weight:700}}.italic{{font-size:1em;font-style:italic;font-weight:400}}.box{{display:inline-block;width:{width}ch;max-width:100%;vertical-align:top;margin:0}}.code{{background-color:#f4f5f6;box-shadow:0 0 0 .5rem #f4f5f6;z-layer:-1;border-radius:4px}}.toc-row{{display:flex;width:100%}}.toc-line{{flex-grow:1;height:.5rem;border-bottom:1px solid #ccc;margin:0 .5rem}}div{{margin:0;padding:0;border:0;box-sizing:border-box}}p{{margin:0;padding:0;border:0;box-sizing:border-box}}</style>",
            util::escape(title)
        );
        final_html.push_str("</head><body><div class=\"main\">");
        final_html.push_str(&content);
        final_html.push_str("</div></body></html>");

        final_html
    }
}
