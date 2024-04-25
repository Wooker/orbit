/// Resource interface to SoC peripherals, provided by either HAL or PAC
pub trait Resource: Sized {}

pub struct Kernel<R>
where
    R: Resource,
{
    pub(crate) resources: R,
}

impl<R> Kernel<R>
where
    R: Resource,
{
    pub fn new(resources: R) -> Self {
        Self { resources }
    }

    pub fn get_resources(&self) -> &R {
        &self.resources
    }
}

trait Application {}

trait ResourceManagement<Res, App>
where
    Res: Resource,
    App: Application,
{
    type Error;
    fn claim(res: Res, who: App) -> Result<(), Self::Error>;
}

enum KernelError {}
impl<'res, Res, App> ResourceManagement<Res, App> for Kernel<Res>
where
    Res: Resource,
    App: Application,
{
    type Error = KernelError;

    fn claim(res: Res, who: App) -> Result<(), Self::Error> {
        todo!()
    }
}
