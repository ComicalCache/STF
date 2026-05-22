pub struct Document<Page> {
    pub cover: Option<Page>,
    pub pages: Vec<Page>,
}

impl<Page> Document<Page> {
    pub fn new() -> Self { Document { cover: None, pages: Vec::new() } }
}
