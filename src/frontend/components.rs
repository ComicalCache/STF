pub trait Component<Context, Instruction> {
    /// The Component configures something in the Context and/or itself.
    fn configure(&mut self, _: &mut Context) {}

    /// The Component generates its contents based on the current Context and
    /// it's own data.
    fn generate(&mut self, _: &mut Context) -> Option<Vec<Instruction>> { None }

    /// The Component modifies the Document _after_ all generation has happened
    /// based on the current Context and it's own data.
    fn modify(&mut self, _: &mut Context) {}
}
