use std::fmt::Write;

use unicode_segmentation::UnicodeSegmentation;

use crate::{stf::Tag, util};

#[derive(Default)]
struct Header<'s> {
    date: Vec<&'s str>,
    title: Vec<&'s str>,
}

#[derive(Default)]
pub struct Txt<'s> {
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

impl<'s> Txt<'s> {
    pub fn generate(tags: impl Iterator<Item = Tag<'s>> + Clone + 's, line_width: usize, page_rows: usize) -> String {
        let mut txt =
            Txt { line_width, page_rows, content_rows: page_rows - 2, page_number_rows: 2, ..Default::default() };

        let mut buff = String::new();
        let mut toc_idx: Option<usize> = None;
        let mut toc_curr_page_row: Option<usize> = None;

        for tag in tags {
            match tag {
                Tag::Cover { .. } => txt.cover(&mut buff, &tag),
                Tag::Header { .. } => txt.header(&tag),
                Tag::TableOfContents => {
                    toc_idx = Some(buff.len());
                    toc_curr_page_row = Some(txt.curr_page_row);
                }
                Tag::Linebreak => txt.linebreak(&mut buff, true, true, true),
                Tag::Pagebreak => txt.pagebreak(&mut buff, true, true, true),
                Tag::Heading { .. } => txt.heading(&mut buff, tag),
                Tag::Text { .. } => txt.text(&mut buff, &tag),
                Tag::Code { .. } => txt.code(&mut buff, &tag),
            }
        }
        txt.pagebreak(&mut buff, true, false, false);

        if let (Some(idx), Some(curr_page_row)) = (toc_idx, toc_curr_page_row) {
            txt.curr_page_row = curr_page_row;

            let tail = buff.split_off(idx);
            txt.table_of_contents(&mut buff);
            buff.push_str(&tail);
        }

        buff
    }

    fn cover(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Cover { title, author, date, notes } = tag else { unreachable!() };

        // The title starts 25% down the page.
        for _ in 0..(self.page_rows as f32 * 0.25) as usize {
            buff.push('\n');
            self.curr_page_row += 1;
        }

        for line in util::wrap_paragraph(title, self.line_width) {
            let _ = writeln!(buff, "{line:^width$}", width = self.line_width);
            self.curr_page_row += 1;
        }

        // One line padding between title and author.
        buff.push('\n');
        self.curr_page_row += 1;

        for line in util::wrap_paragraph(author, self.line_width) {
            let _ = writeln!(buff, "{line:^width$}", width = self.line_width);
            self.curr_page_row += 1;
        }

        // The date is right bellow the author.
        for line in util::wrap_paragraph(date, self.line_width) {
            let _ = writeln!(buff, "{line:^width$}", width = self.line_width);
            self.curr_page_row += 1;
        }

        let mut notes_lines = 0;
        let notes = util::wrap_paragraph(notes, self.line_width).fold(String::new(), |mut acc, line| {
            let _ = writeln!(acc, "{line:^width$}", width = self.line_width);
            notes_lines += 1;

            acc
        });

        // Pad the page until the notes will leave exactly five empty trailing line on the page.
        let notes_pad = self.page_rows - self.curr_page_row - notes_lines - 5;
        for _ in 0..notes_pad {
            buff.push('\n');
            self.curr_page_row += 1;
        }

        buff.push_str(&notes);
        self.curr_page_row += notes_lines;

        // Three lines padding to the footer.
        for _ in 0..3 {
            buff.push('\n');
            self.curr_page_row += 1;
        }

        // Add a pagebreak.
        self.pagebreak(buff, false, true, true);
    }

    fn header(&mut self, tag: &Tag<'s>) {
        let Tag::Header { date, title } = tag else { unreachable!() };

        let date: Vec<&'s str> = util::wrap_paragraph(date, self.line_width).collect();
        let title: Vec<&'s str> = util::wrap_paragraph(title, self.line_width).collect();
        let header = Header { date, title };

        self.header_rows = header.date.len() + header.title.len() + 1;
        self.content_rows -= self.header_rows;

        self.header = Some(header);
    }

    fn linebreak(&mut self, buff: &mut String, write_page_number: bool, increase_page_count: bool, write_header: bool) {
        buff.push('\n');
        self.curr_page_row += 1;

        if self.curr_page_row == self.header_rows + self.content_rows {
            self.pagebreak(buff, write_page_number, increase_page_count, write_header);
        }
    }

    fn pagebreak(&mut self, buff: &mut String, write_page_number: bool, increase_page_count: bool, write_header: bool) {
        while self.curr_page_row < self.page_rows - self.page_number_rows {
            buff.push('\n');
            self.curr_page_row += 1;
        }

        // Add one line of padding above the page count.
        buff.push('\n');

        if write_page_number {
            let _ = writeln!(buff, "{:>width$}", format!("[Page {}]", self.curr_page_number), width = self.line_width);
        } else {
            buff.push('\n');
        }

        // Inserts form feed and newline as pagebreak.
        buff.push('\x0c');
        buff.push('\n');
        self.curr_page_row = 0;

        if increase_page_count {
            self.curr_page_number += 1;
        }

        if write_header && let Some(header) = &self.header {
            // TODO: add error if write_heder == true but no header was specified.

            for line in &header.date {
                let _ = writeln!(buff, "{line}");
                self.curr_page_row += 1;
            }
            for line in &header.title {
                let _ = writeln!(buff, "{line:>width$}", width = self.line_width);
                self.curr_page_row += 1;
            }

            // Add a line of padding bellow the header.
            buff.push('\n');
            self.curr_page_row += 1;
        }
    }

    fn heading(&mut self, buff: &mut String, tag: Tag<'s>) {
        let Tag::Heading { content } = tag else { unreachable!() };

        // Store the page number of where the heading starts.
        let page = self.curr_page_number;

        for line in util::wrap_paragraph(&content, self.line_width) {
            let _ = write!(buff, "{line:^width$}", width = self.line_width);
            self.linebreak(buff, true, true, true);
        }

        self.headings.push((content, page));

        // Padding bellow.
        self.linebreak(buff, true, true, true);
    }

    fn text(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Text { content } = tag else { unreachable!() };

        for line in util::wrap_paragraph(content, self.line_width) {
            buff.push_str(line);
            self.linebreak(buff, true, true, true);
        }
    }

    fn code(&mut self, buff: &mut String, tag: &Tag<'_>) {
        let Tag::Code { content } = tag else { unreachable!() };

        for line in util::wrap_code(content, self.line_width) {
            buff.push_str(line);
            self.linebreak(buff, true, true, true);
        }
    }

    fn table_of_contents(&mut self, buff: &mut String) {
        if self.headings.is_empty() {
            return;
        }

        let _ = write!(buff, "{:^width$}", "Table Of Contents", width = self.line_width);
        self.linebreak(buff, false, false, true);

        let max_page_num_width = self.headings.last().unwrap().1.checked_ilog10().unwrap_or(0) as usize + 1;

        let mut last_line = "";
        // Cheap clone because of &str.
        for (title, page) in &self.headings.clone() {
            // Minus three for enough space for at least one dot.
            for line in util::wrap_paragraph(title, self.line_width - max_page_num_width - 3) {
                self.linebreak(buff, false, false, true);

                let _ = write!(buff, "{line}");

                last_line = line;
            }

            let last_line_graphemes = last_line.graphemes(true).count();
            let _ = write!(
                buff,
                " {:.>width$} {page:>max_page_num_width$}",
                "",
                width = self.line_width - last_line_graphemes - max_page_num_width - 2
            );
        }

        self.pagebreak(buff, false, false, true);
    }
}
