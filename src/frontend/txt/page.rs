use unicode_segmentation::UnicodeSegmentation;

pub struct Page {
    /// Width in graphemes.
    pub width: usize,
    /// Maximum number of lines per page (including header and footer).
    pub max_lines: usize,

    /// Lines of the header.
    header: Vec<String>,
    /// Minimum number lines of the header.
    pub min_header: usize,
    /// Amount of lines for header padding.
    pub header_padding: usize,

    /// Lines of the body.
    body: Vec<String>,

    /// Lines of the footnotes.
    footnotes: Vec<String>,
    /// Amount of lines for footnotes padding.
    pub footnotes_padding: usize,

    /// Lines of the footer.
    footer: Vec<String>,
    /// Minimum number lines of the footer.
    pub min_footer: usize,
    /// Amount of lines for footer padding.
    pub footer_padding: usize,
}

impl Page {
    pub const fn new(
        width: usize, max_lines: usize, min_header: usize, header_padding: usize, footnotes_padding: usize,
        min_footer: usize, footer_padding: usize,
    ) -> Self {
        Self {
            width,
            max_lines,
            header: Vec::new(),
            min_header,
            header_padding,
            body: Vec::new(),
            footnotes: Vec::new(),
            footnotes_padding,
            footer: Vec::new(),
            min_footer,
            footer_padding,
        }
    }

    pub fn header(&self) -> &[String] { &self.header }
    pub fn body(&self) -> &[String] { &self.body }
    pub fn footnotes(&self) -> &[String] { &self.footnotes }
    pub fn footer(&self) -> &[String] { &self.footer }

    /// Pushes a line into the header, asserting the grapheme width invariant.
    pub fn push_header(&mut self, line: String) {
        assert!(line.graphemes(true).count() <= self.width);
        assert!(self.lines() < self.max_lines);
        assert!(!line.contains('\n'));

        self.header.push(line);
    }

    /// Pushes a line into the body, asserting the grapheme width invariant.
    pub fn push_body(&mut self, line: String) {
        assert!(line.graphemes(true).count() <= self.width);
        assert!(self.lines() < self.max_lines);
        assert!(!line.contains('\n'));

        self.body.push(line);
    }

    /// Pushes a line into the footnotes, asserting the grapheme width
    /// invariant.
    pub fn push_footnote(&mut self, line: String) {
        assert!(line.graphemes(true).count() <= self.width);
        assert!(self.lines() < self.max_lines);
        assert!(!line.contains('\n'));

        self.footnotes.push(line);
    }

    /// Pushes a line into the footer, asserting the grapheme width invariant.
    pub fn push_footer(&mut self, line: String) {
        assert!(line.graphemes(true).count() <= self.width);
        assert!(self.lines() < self.max_lines);
        assert!(!line.contains('\n'));

        self.footer.push(line);
    }

    /// Returns the number of filled lines on the page.
    pub fn lines(&self) -> usize {
        self.header.len().max(self.min_header)
            + self.header_padding
            + self.body.len()
            + self.footnotes_padding
            + self.footnotes.len()
            + self.footer_padding
            + self.footer.len().max(self.min_footer)
    }

    /// Returns whether this amount of new lines fits onto the page.
    pub fn fits(&self, new_lines: usize) -> bool { self.lines() + new_lines <= self.max_lines }
}
