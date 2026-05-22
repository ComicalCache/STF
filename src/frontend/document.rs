pub struct Document<Page> {
    pub cover: Option<Page>,
    pub pages: Vec<Page>,
}

impl<Page> Document<Page> {
    pub const fn new() -> Self { Self { cover: None, pages: Vec::new() } }
}
