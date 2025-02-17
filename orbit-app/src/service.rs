pub trait Service<const PRIORITY: u8> {
    #[link_section = ".apps"]
    fn serve(&self);
}
