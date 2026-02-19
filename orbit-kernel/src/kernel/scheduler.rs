use core::pin::Pin;

use alloc::{boxed::Box, collections::linked_list::LinkedList};

use crate::task::Task;

pub(super) struct Scheduler<'s> {
    list: LinkedList<Pin<Box<Task<'s>>>>,
}

impl<'s> Scheduler<'s> {
    pub(super) fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub(super) fn current(&self) -> Option<&Pin<Box<Task<'s>>>> {
        self.list.front()
    }

    pub(super) fn pop(&mut self) -> Option<Pin<Box<Task<'s>>>> {
        self.list.pop_front()
    }

    pub(super) fn add(&mut self, task: Pin<Box<Task<'s>>>) {
        let mut front = self.list.cursor_front_mut();

        if let Some(current) = front.current()
            && current.priority > task.priority
        {
            let mut cursor = self.list.cursor_front_mut();
            while let Some(current) = cursor.current()
                && task.priority <= current.priority
            {
                cursor.move_next();
            }
            cursor.insert_before(task);
        } else {
            front.insert_before(task);
        }
    }
}
