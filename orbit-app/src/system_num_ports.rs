use orbit_common::{app_heap, app_stack};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::syscall::SysCall;

use crate::syscall;

app_heap!(0);
app_stack!(64);

#[orbit_app()]
pub struct SystemNumPorts {}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}
#[orbit_impl]
impl SystemNumPorts {
    #[app_init("systemnumports")]
    pub fn init(&mut self) {}

    #[app_interrupt("systemnumports")]
    pub fn interrupt(&mut self) {}

    #[app_main("systemnumports")]
    pub fn main(&mut self) -> Output {
        self._buf.flush();
        syscall!(SysCall::NumPorts);
        let num_ports = usize::from_le_bytes(unsafe {
            self._buf
                .read()
                .unwrap_unchecked()
                .split_last()
                .unwrap_unchecked()
                .1
                .try_into()
                .unwrap_unchecked()
        });

        "\x01systemnumports \0"
            .bytes()
            .for_each(|ch| self._buf.push(ch));
        syscall!(SysCall::SendAll);

        if num_ports == 1 {
            Output([num_ports as u8])
        } else {
            syscall!(SysCall::Await);
            let system_num_ports = unsafe {
                self._buf
                    .read()
                    .unwrap_unchecked()
                    .split_last()
                    .unwrap_unchecked()
                    .1[0]
            } + 1;
            Output([system_num_ports as u8])
        }
    }
}
