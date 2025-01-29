pub trait Application<const PRIORITY: u8> {
    #[link_section = ".apps"]
    fn main(&self);
}
