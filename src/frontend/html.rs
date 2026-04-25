use std::fmt::Write;

use crate::{stf::Tag, util};

#[derive(Default)]
struct Header<'s> {
    date: Vec<&'s str>,
    title: Vec<&'s str>,
}

#[derive(Default)]
pub struct Html<'s> {
    /// Width of a line in Unicode graphemes.
    line_width: usize,
    /// Total available rows in a page including headers, contents and page numbers.
    page_rows: usize,
    header_rows: usize,
    content_rows: usize,
    page_number_rows: usize,

    /// Absolute current page row.
    curr_page_row: usize,

    curr_page_number: usize,
    header: Option<Header<'s>>,

    headings: Vec<(String, usize)>,
}

impl<'s> Html<'s> {
    pub fn generate(
        title: &str, tags: impl Iterator<Item = Tag<'s>> + Clone + 's, line_len: usize, page_len: usize,
    ) -> String {
        let mut html = Html {
            line_width: line_len,
            page_rows: page_len,
            content_rows: page_len - 2,
            page_number_rows: 2,
            ..Default::default()
        };

        let mut buff = String::new();

        buff.push_str("<!DOCTYPE html><html><head><meta charset=\"UTF-8\">");
        let _ = writeln!(buff, "<title>{}</title>", util::escape(title));
        let _ = writeln!(
            buff,
            "<style>body{{font-family:monospace}}.main{{max-width:{width}ch;width:100%;margin:0 auto}}.align-center{{text-align:center}}.align-right{{text-align:right}}.pagebreak{{height:0;border-top:1px solid #ccc;width:100%}}.heading{{font-size:1em;font-weight:700}}.italic{{font-size:1em;font-style:italic;font-weight:400}}.box{{display:inline-block;width:{width}ch;max-width:100%;vertical-align:top;margin:0}}.code{{background-color:#f4f5f6;box-shadow:0 0 0 .5rem #f4f5f6;z-layer:-1;border-radius:4px}}.toc-row{{display:flex;width:100%}}.toc-line{{flex-grow:1;height:.5rem;border-bottom:1px solid #ccc;margin:0 .5rem}}div{{margin:0;padding:0;border:0;box-sizing:border-box}}p{{margin:0;padding:0;border:0;box-sizing:border-box}}</style>",
            width = html.line_width
        );
        buff.push_str("<div class=\"main\">");

        let mut toc_idx: Option<usize> = None;
        let mut toc_curr_page_row: Option<usize> = None;

        for tag in tags {
            match tag {
                Tag::Cover { .. } => html.cover(&mut buff, &tag),
                Tag::HeaderConfig { .. } => html.header_config(&tag),
                Tag::TableOfContents => {
                    toc_idx = Some(buff.len());
                    toc_curr_page_row = Some(html.curr_page_row);
                }
                Tag::Header => html.header(&mut buff),
                Tag::Linebreak => html.linebreak(&mut buff, true, true, true),
                Tag::Pagebreak => html.pagebreak(&mut buff, true, true, true),
                Tag::Heading { .. } => html.heading(&mut buff, tag),
                Tag::Text { .. } => html.text(&mut buff, &tag),
                Tag::Code { .. } => html.code(&mut buff, &tag),
                Tag::Link { .. } => html.link(&mut buff, &tag),
            }
        }
        html.pagebreak(&mut buff, true, false, false);

        if let (Some(idx), Some(curr_page_row)) = (toc_idx, toc_curr_page_row) {
            html.curr_page_row = curr_page_row;

            let tail = buff.split_off(idx);
            html.table_of_contents(&mut buff);
            buff.push_str(&tail);
        }

        buff.push_str("</div></body></html>");
        buff
    }

    fn cover(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Cover { title, author, date, notes } = tag else { unreachable!() };

        // The title starts 25% down the page.
        for _ in 0..(self.page_rows as f32 * 0.25) as usize {
            buff.push_str("<br>");
            self.curr_page_row += 1;
        }

        for line in util::wrap_paragraph(title, self.line_width) {
            let _ = write!(buff, "<p class=\"align-center heading\">{}</p>", util::escape(line));
            self.curr_page_row += 1;
        }

        // One line padding between title and author.
        buff.push_str("<br>");
        self.curr_page_row += 1;

        for line in util::wrap_paragraph(author, self.line_width) {
            let _ = write!(buff, "<p class=\"align-center italic\">{}</p>", util::escape(line));
            self.curr_page_row += 1;
        }

        // The date is right bellow the author.
        for line in util::wrap_paragraph(date, self.line_width) {
            let _ = write!(buff, "<p class=\"align-center\">{}</p>", util::escape(line));
            self.curr_page_row += 1;
        }

        let mut notes_lines = 0;
        let notes = util::wrap_paragraph(notes, self.line_width).fold(String::new(), |mut acc, line| {
            let _ = write!(acc, "<p class=\"align-center\">{}</p>", util::escape(line));
            notes_lines += 1;

            acc
        });

        // Pad the page until the notes will leave exactly five empty trailing line on the page.
        let notes_pad = self.page_rows - self.curr_page_row - notes_lines - 5;
        for _ in 0..notes_pad {
            buff.push_str("<br>");
            self.curr_page_row += 1;
        }

        buff.push_str(&notes);
        self.curr_page_row += notes_lines;

        // Three lines padding to the footer.
        for _ in 0..3 {
            buff.push_str("<br>");
            self.curr_page_row += 1;
        }

        // Add a pagebreak.
        self.pagebreak(buff, false, true, true);
    }

    fn header_config(&mut self, tag: &Tag<'s>) {
        let Tag::HeaderConfig { date, title } = tag else { unreachable!() };

        let date: Vec<&'s str> = util::wrap_paragraph(date, self.line_width).collect();
        let title: Vec<&'s str> = util::wrap_paragraph(title, self.line_width).collect();
        let header = Header { date, title };

        self.header_rows = header.date.len() + header.title.len() + 1;
        self.content_rows -= self.header_rows;

        self.header = Some(header);
    }

    fn table_of_contents(&mut self, buff: &mut String) {
        if self.headings.is_empty() {
            return;
        }

        buff.push_str("<span class=\"box heading align-center\">Table Of Contents</span>");
        self.linebreak(buff, false, false, true);

        let max_page_num_width = self.headings.last().unwrap().1.checked_ilog10().unwrap_or(0) as usize + 1;

        let mut pre_last_line: usize = 0;
        let mut last_line = "";
        // Cheap clone because of &str.
        for (title, page) in &self.headings.clone() {
            // Minus three for enough space for at least one dot.
            for line in util::wrap_paragraph(title, self.line_width - max_page_num_width - 3) {
                self.linebreak(buff, false, false, true);

                pre_last_line = buff.len();
                let _ = write!(buff, "<span class=\"box\">{}</span>", util::escape(line));

                last_line = line;
            }

            buff.truncate(pre_last_line);

            let _ = write!(
                buff,
                "<div class=\"box\"><div class=\"toc-row\"><span>{}</span><span class=\"toc-line\"></span><span>{page:>max_page_num_width$}</span></div></div>",
                util::escape(last_line),
            );
        }

        self.pagebreak(buff, false, false, true);
    }

    fn header(&mut self, buff: &mut String) {
        if let Some(header) = &self.header {
            // TODO: add error if write_heder == true but no header was specified.

            for line in &header.date {
                let _ = write!(buff, "<p>{}</p>", util::escape(line));
                self.curr_page_row += 1;
            }
            for line in &header.title {
                let _ = write!(buff, "<p class=\"align-right italic\">{}</p>", util::escape(line));
                self.curr_page_row += 1;
            }

            // Add a line of padding bellow the header.
            buff.push_str("<br>");
            self.curr_page_row += 1;
        }
    }

    fn linebreak(&mut self, buff: &mut String, write_page_number: bool, increase_page_count: bool, write_header: bool) {
        buff.push_str("<br>");
        self.curr_page_row += 1;

        if self.curr_page_row == self.header_rows + self.content_rows {
            self.pagebreak(buff, write_page_number, increase_page_count, write_header);
        }
    }

    fn pagebreak(&mut self, buff: &mut String, write_page_number: bool, increase_page_count: bool, write_header: bool) {
        while self.curr_page_row < self.page_rows - self.page_number_rows {
            buff.push_str("<br>");
            self.curr_page_row += 1;
        }

        // Add one line of padding above the page count.
        buff.push_str("<br>");

        if write_page_number {
            let _ = write!(buff, "<p class=\"align-right\">[Page {}]</p>", self.curr_page_number);
        } else {
            buff.push_str("<br>");
        }

        buff.push_str("<br><div class=\"pagebreak\"></div><br>");
        self.curr_page_row = 0;

        if increase_page_count {
            self.curr_page_number += 1;
        }

        if write_header {
            self.header(buff);
        }
    }

    fn heading(&mut self, buff: &mut String, tag: Tag<'s>) {
        let Tag::Heading { content } = tag else { unreachable!() };

        // Store the page number of where the heading starts.
        let page = self.curr_page_number;

        for line in util::wrap_paragraph(&content, self.line_width) {
            let _ = write!(buff, "<span class=\"box heading align-center\">{}</span>", util::escape(line));
            self.linebreak(buff, true, true, true);
        }

        self.headings.push((content, page));

        // Padding bellow.
        self.linebreak(buff, true, true, true);
    }

    fn text(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Text { content } = tag else { unreachable!() };

        for line in util::wrap_paragraph(content, self.line_width) {
            let _ = write!(buff, "<span>{}</span>", util::escape(line));
            self.linebreak(buff, true, true, true);
        }
    }

    fn code(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Code { content } = tag else { unreachable!() };

        // Box to avoid leading <br>, but requires manual <br> at the end.
        buff.push_str("<pre class=\"box code\">");
        for line in util::wrap_code(content, self.line_width) {
            let _ = write!(buff, "<code>{}</code>", util::escape(line));
            self.linebreak(buff, true, true, true);
        }
        buff.push_str("</pre><br>");
    }

    fn link(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Link { url, abbrev, content } = tag else { unreachable!() };

        let text = format!("{} ({})", content, abbrev);

        // Plus two for the space and opening bracket.
        let abbrev_start = content.len() + 2;
        let abbrev_end = abbrev_start + abbrev.len();

        for line in util::wrap_paragraph(&text, self.line_width) {
            let offset = line.as_ptr() as usize - text.as_ptr() as usize;

            let start = (abbrev_start - offset).min(line.len());
            let stop = (abbrev_end - offset).min(line.len());

            let before = &line[..start];
            let inside = &line[start..stop];
            let after = &line[stop..];

            buff.push_str("<span>");
            if !before.is_empty() {
                buff.push_str(&util::escape(before));
            }
            if !inside.is_empty() {
                let _ = write!(buff, "<a href=\"{}\">{}</a>", url, util::escape(inside));
            }
            if !after.is_empty() {
                buff.push_str(&util::escape(after));
            }
            buff.push_str("</span>");
            self.linebreak(buff, true, true, true);
        }
    }
}
