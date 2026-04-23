use crate::stf::Tag;

#[derive(Default)]
pub struct Html {
    page_number: usize,
}

impl Html {
    pub fn generate<'s>(
        title: &str, tags: impl Iterator<Item = Tag<'s>> + Clone + 's, line_len: usize, page_len: usize,
    ) -> String {
        let mut ret = String::new();

        ret
    }
}
