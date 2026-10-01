use core::pin::Pin;

use alloc::{boxed::Box, collections::linked_list::LinkedList};

use crate::task::Task;

pub(super) struct Scheduler {
    list: LinkedList<Pin<Box<Task>>>,
}

impl Scheduler {
    pub(super) fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    pub(super) fn current(&self) -> Option<&Pin<Box<Task>>> {
        self.list.front()
    }

    pub(super) fn current_mut(&mut self) -> Option<&mut Pin<Box<Task>>> {
        self.list.front_mut()
    }

    pub(super) fn pop(&mut self) -> Option<Pin<Box<Task>>> {
        self.list.pop_front()
    }

    pub(super) fn list(&self) -> &LinkedList<Pin<Box<Task>>> {
        &self.list
    }

    pub(super) fn list_mut(&mut self) -> &mut LinkedList<Pin<Box<Task>>> {
        &mut self.list
    }

    /// Add a task to scheduler list.
    /// Lower priority places the task closer to the front
    /// of the list.
    pub(super) fn add(&mut self, task: Pin<Box<Task>>) {
        let mut cursor = self.list.cursor_front_mut();

        while let Some(current) = cursor.current()
            && current.header.priority <= task.header.priority
        {
            cursor.move_next();
        }
        cursor.insert_before(task);
    }
}
