pub mod data;
pub mod driver;
pub mod invokable;
pub mod named;
pub mod systems;

#[derive(Debug, Default, Copy, Clone)]
pub struct Container<'c> {
    pub named: Option<named::NamedContainer<'c>>,
    pub invokable: Option<invokable::InvokableContainer>,
    pub data: Option<data::DataContainer>,
    pub driver: Option<driver::DriverContainer>,
    pub systems: Option<systems::SystemContainer>,
}

pub trait Containable<'c> {
    fn to_container(&self) -> Container<'c>;
}
