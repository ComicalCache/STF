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

use crate::{
    frontend::{
        components::Component,
        document::Document,
        txt::{
            code::Code, cover::Cover, header_config::HeaderConfig, heading::Heading, linebreak::Linebreak, link::Link,
            page::Page, pagebreak::Pagebreak, table_of_contents::TableOfContents, text::Text,
        },
    },
    stf::Tag,
};

struct Header {
    left: Vec<String>,
    right: Vec<String>,
}

pub struct TxtContext {
    /// Width in graphemes.
    width: usize,
    /// Maximum number of lines per page (including header and footer).
    max_lines: usize,

    header: Option<Header>,

    /// Headings on page with title.
    headings: Vec<(usize, String)>,

    doc: Document<Page>,
}

impl TxtContext {
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
                page.push_header(format!("{line:>width$}", width = self.width));
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

pub enum TxtInstruction {
    /// A single line.
    Line(String),
    /// A block that must not be broken by a pagebreak.
    Block(Vec<String>),
    /// A paragraph that may not have a lone line on the previous or following
    /// page.
    Paragraph(Vec<String>),
    /// Creates a new page.
    Pagebreak,
    /// Adds a line of padding.
    Padding,
    /// Registers a Table of Contents entry at the current page.
    RegisterToC(String),
}

type TxtComponent = Box<dyn Component<TxtContext, TxtInstruction>>;

pub struct Txt {
    components: Vec<TxtComponent>,
    ctx: TxtContext,
}

impl Txt {
    #[allow(clippy::too_many_lines)]
    pub fn generate(tags: impl Iterator<Item = Tag>, width: usize, max_lines: usize) -> String {
        let mut txt = Self {
            components: tags
                .map(|tag| match tag {
                    Tag::Cover { .. } => Box::new(Cover::new(tag)) as TxtComponent,
                    Tag::HeaderConfig { .. } => Box::new(HeaderConfig::new(tag)) as TxtComponent,
                    Tag::TableOfContents => Box::new(TableOfContents::new(&tag)) as TxtComponent,
                    Tag::Linebreak => Box::new(Linebreak::new(&tag)) as TxtComponent,
                    Tag::Pagebreak => Box::new(Pagebreak::new(&tag)) as TxtComponent,
                    Tag::Heading { .. } => Box::new(Heading::new(tag)) as TxtComponent,
                    Tag::Text(_) => Box::new(Text::new(tag)) as TxtComponent,
                    Tag::Code(_) => Box::new(Code::new(tag)) as TxtComponent,
                    Tag::Link { .. } => Box::new(Link::new(tag)) as TxtComponent,
                })
                .collect(),
            ctx: TxtContext::new(width, max_lines),
        };

        // First pass for Components that need setup.
        for component in &mut txt.components {
            component.configure(&mut txt.ctx);
        }

        // Second pass for Components that can write their contents.
        let mut components = std::mem::take(&mut txt.components);
        for component in &mut components {
            let Some(instructions) = component.generate(&mut txt.ctx) else {
                continue;
            };

            for instruction in instructions {
                match instruction {
                    TxtInstruction::Line(data) => {
                        let page = txt.ctx.last_page();

                        if page.fits(1) {
                            page.push_body(data);
                        } else {
                            txt.ctx.break_page().push_body(data);
                        }
                    }
                    TxtInstruction::Block(lines) => {
                        assert!(lines.len() <= max_lines);

                        let page = txt.ctx.last_page();

                        if page.fits(lines.len()) {
                            for line in lines {
                                page.push_body(line);
                            }
                        } else {
                            let page = txt.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        }
                    }
                    TxtInstruction::Paragraph(mut lines) => {
                        let page = txt.ctx.last_page();

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
                            let page = txt.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        } else if page.fits(1) {
                            // Avoid having a single leading line cause a page
                            // break, break immediately.

                            let page = txt.ctx.break_page();
                            for line in lines {
                                page.push_body(line);
                            }
                        }
                    }
                    TxtInstruction::Pagebreak => {
                        txt.ctx.break_page();
                    }
                    TxtInstruction::Padding => {
                        let page = txt.ctx.last_page();

                        if !page.body().is_empty() {
                            if page.fits(1) {
                                page.push_body(String::new());
                            } else {
                                txt.ctx.break_page();
                            }
                        }
                    }
                    TxtInstruction::RegisterToC(title) => txt.ctx.headings.push((txt.ctx.doc.pages.len(), title)),
                }
            }
        }
        txt.components = components;

        // Add page numbers generated pages.
        let width = txt.ctx.width;
        for (idx, page) in txt.ctx.doc.pages.iter_mut().enumerate() {
            let page_nr = format!("[Page {}]", idx + 1);
            page.push_footer(format!("{page_nr:>width$}"));
        }

        // Third pass for Components that need information added by other, already
        // lay-outed Components.
        for component in &mut txt.components {
            component.modify(&mut txt.ctx);
        }

        // Assemble the Document into a String.
        let assemble = |page: &Page| {
            let mut lines = Vec::with_capacity(page.max_lines);

            // Remove trailing whitespace.
            lines.extend(page.header().iter().map(|line| String::from(line.trim_end())));
            // Pad to the minimum header lines.
            for _ in page.header().len()..page.min_header {
                lines.push(String::new());
            }
            for _ in 0..page.header_padding {
                lines.push(String::new());
            }

            // Remove trailing whitespace.
            lines.extend(page.body().iter().map(|line| String::from(line.trim_end())));

            // Padding.
            for _ in 0..page.max_lines.saturating_sub(page.lines()) {
                lines.push(String::new());
            }

            for _ in 0..page.footnotes_padding {
                lines.push(String::new());
            }
            // Remove trailing whitespace.
            lines.extend(page.footnotes().iter().map(|line| String::from(line.trim_end())));

            for _ in 0..page.footer_padding {
                lines.push(String::new());
            }
            // Pad to the minimum footer lines.
            for _ in page.footer().len()..page.min_footer {
                lines.push(String::new());
            }
            // Remove trailing whitespace.
            lines.extend(page.footer().iter().map(|line| String::from(line.trim_end())));

            lines.join("\n")
        };

        let cover = txt.ctx.doc.cover.map(|page| assemble(&page) + "\n\x0c\n").unwrap_or_default();
        let pages = txt.ctx.doc.pages.iter().map(assemble).collect::<Vec<String>>().join("\n\x0c\n");

        cover + &pages
    }
}
