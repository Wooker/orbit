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

    pub(super) fn current_mut(&mut self) -> Option<&mut Pin<Box<Task<'s>>>> {
        self.list.front_mut()
    }

    pub(super) fn pop(&mut self) -> Option<Pin<Box<Task<'s>>>> {
        self.list.pop_front()
    }

    pub(super) fn list(&self) -> &LinkedList<Pin<Box<Task<'s>>>> {
        &self.list
    }

    pub(super) fn list_mut(&mut self) -> &mut LinkedList<Pin<Box<Task<'s>>>> {
        &mut self.list
    }

    pub(super) fn add(&mut self, task: Pin<Box<Task<'s>>>) {
        let mut front = self.list.cursor_front_mut();

        if let Some(current) = front.current()
            && current.header.priority > task.header.priority
        {
            let mut cursor = self.list.cursor_front_mut();
            while let Some(current) = cursor.current()
                && task.header.priority <= current.header.priority
            {
                cursor.move_next();
            }
            cursor.insert_before(task);
        } else {
            front.insert_before(task);
        }
    }
}
