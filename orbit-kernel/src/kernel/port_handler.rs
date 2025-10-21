use core::mem::MaybeUninit;

use crate::{
    RingbufType,
    application_container::AppContainer,
    kernel::{APPS, Message, RunApplication, Terminate},
    port::Port,
};

#[inline(never)]
pub(super) fn handle_invoke(
    port: &mut Port,
    msg: &[u8],
    apps: &mut [MaybeUninit<AppContainer>; APPS],
    running: &mut Option<usize>,
) -> RunApplication {
    let (name, arg) = if let Some((name, arg)) =
        msg.split_once(|p| *p == <RingbufType as Terminate>::termination())
    {
        (name, Some(arg))
    } else {
        (msg, None)
    };

    if let Some((app_index, _maybe_app)) = apps.iter().enumerate().find(|(_, app)| {
        let app = unsafe { app.assume_init_read() };
        let app_name = app.name();
        app_name.as_bytes().eq(name)
    }) {
        let app = unsafe { apps.get_unchecked_mut(app_index).assume_init_mut() };

        // Flush the application buffer
        let app_buf = app.buf();
        app_buf.flush();
        if let Some(arg) = arg {
            // Write command arguments after the space to
            // the application buffer
            arg.iter().for_each(|ch| app_buf.push(*ch));
            app_buf.fill();

            *running = Some(app_index);
            RunApplication::Main
        } else {
            *running = Some(app_index);
            RunApplication::Main
        }
    } else {
        port.write_str(&[Message::Unknown.into(), Message::Invoke.into()]);
        port.msg -= 1;
        RunApplication::None
    }
}
