pub trait Application {
    #[inline(never)]
    // #[link_section = ".apps"]
    fn main(&self);
}
