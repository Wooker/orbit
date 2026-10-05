#[derive(Debug, Clone, Copy)]
pub struct NamedContainer<'a> {
    name: &'a str,
}

impl<'a> NamedContainer<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

pub trait NamedAuto<'a> {
    const NAME: &'a str;

    #[inline(never)]
    fn to_named_container(&self) -> NamedContainer<'a>
    where
        Self: Sized,
    {
        NamedContainer::new(Self::NAME)
    }
}
