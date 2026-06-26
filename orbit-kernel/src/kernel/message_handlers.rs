use alloc::vec::Vec;
use spaceport::{
    message::Message,
    packet::{MAX_BUFFER_LENGTH, Packet},
    types::Flags,
};

use crate::{application_container::AppContainer, kernel::Scheduler, task::Task};

pub(super) fn handle_invoke<'kernel, 'old_packet, 'new_packet>(
    apps: &'old_packet mut Vec<AppContainer<'kernel>>,
    drivers: &'old_packet mut Vec<AppContainer<'kernel>>,
    scheduler: &'old_packet mut Scheduler,
    mut packet: Packet<'old_packet>,
) -> Option<Packet<'new_packet>>
where
    'kernel: 'old_packet,
    'kernel: 'new_packet,
    'new_packet: 'old_packet,
{
    let (id, src, dst) = (packet.packet_id + 1, packet.dst, packet.src);

    let divider_index = packet.payload.iter().enumerate().find(|(i, b)| **b == b' ');
    let (name, arg) = divider_index.map_or((packet.payload, None), |index| {
        let (a, b) = packet.payload.split_at(index.0);
        (a, Some(b))
    });

    if let Some(arg) = arg {
        packet.payload = &packet.payload[name.len() + 1..];
    } else {
        packet.payload = &[];
    }

    let mut out = [0u8; MAX_BUFFER_LENGTH];
    if let Some(app) = apps.iter().find(|(app)| app.name().as_bytes().eq(name)) {
        if let Some(task) = Task::new(packet, 1, app.main_addr()) {
            let task_id = task.header.task_id;
            scheduler.add(task);
            None
        } else {
            Some(Packet::new(
                Flags::ERROR,
                id,
                src,
                dst,
                Message::Reply,
                b"Could not create task",
            ))
        }
    } else if let Some(app) = drivers.iter().find(|(app)| app.name().as_bytes().eq(name)) {
        if let Some(mut task) = Task::new(packet, 1, app.main_addr()) {
            let task_id = task.header.task_id;
            task.for_driver(app.driver_struct().unwrap());
            scheduler.add(task);
            None
        } else {
            Some(Packet::new(
                Flags::ERROR,
                id,
                src,
                dst,
                Message::Reply,
                b"Could not create task",
            ))
        }
    } else {
        Some(Packet::new(
            Flags::ERROR,
            id,
            src,
            dst,
            Message::Error,
            b"Unknown name",
        ))
    }
}
