use core::{error::Error, mem::MaybeUninit};

use spaceport::types::Flags;

use crate::{
    application_container::AppContainer,
    kernel::{APPS, RunApplication},
};

pub enum InvokeError {
    UnknownName,
}

#[inline(never)]
pub(super) fn handle_invoke(
    msg: &[u8],
    apps: &mut [MaybeUninit<AppContainer>; APPS],
    running: &mut Option<usize>,
) -> Result<RunApplication, InvokeError> {
    let (name, arg) = if let Some((name, arg)) = msg.split_once(|p| *p == b' ') {
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
            Ok(RunApplication::Main)
        } else {
            *running = Some(app_index);
            Ok(RunApplication::Main)
        }
    } else {
        Err(InvokeError::UnknownName)
    }
}
