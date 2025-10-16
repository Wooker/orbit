use core::mem::MaybeUninit;

use crate::{
    RINGBUF_SIZE, RingbufType,
    application_container::AppContainer,
    kernel::{APPS, Message, RingBuf, RunApplication, TraitBound},
    port::Port,
};

#[inline(never)]
pub(super) fn handle_invoke(
    port: &mut Port,
    msg: &[u8],
    apps: &mut [MaybeUninit<AppContainer>; APPS],
    running: &mut Option<usize>,
) -> RunApplication {
    let mut resp: RingBuf<RINGBUF_SIZE, RingbufType> = RingBuf::default();

    let (name, arg) = if let Some((name, arg)) =
        msg.split_once(|p| *p == <RingbufType as TraitBound>::termination())
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
        if let Some(arg) = arg {
            // arg.iter().for_each(|d| resp.push(*d));
            // port.write_str(&resp.buf);
            // resp.flush();

            // Flush the application buffer
            let app_buf = app.buf();
            app_buf.flush();

            let mut arg_buf: RingBuf<RINGBUF_SIZE, RingbufType> = RingBuf::default();
            // Write command arguments after the space to
            // the application buffer
            arg.iter().for_each(|ch| arg_buf.push(*ch));
            app_buf.buf.copy_from_slice(&arg_buf.buf);
            app_buf.end = RINGBUF_SIZE;
            *running = Some(app_index);
            RunApplication::Main
        } else {
            resp.push(Message::Unknown.into());
            resp.push(Message::Invoke.into());
            name.iter().for_each(|b| resp.push(*b));
            resp.fill();
            port.write_str(&resp.buf);
            port.msg -= 1;
            resp.flush();

            RunApplication::None
        }
    } else {
        port.write_str(b"not found");
        RunApplication::None
    }
}
