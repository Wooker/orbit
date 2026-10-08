use crate::scheduler::State::{Blocked, Waiting};
use alloc::{boxed::Box, collections::linked_list::LinkedList};
use core::pin::Pin;

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    Waiting,
    Blocked(usize),
}

#[derive(Debug)]
pub(super) struct Entry<T: ?Sized> {
    id: usize,
    pub(super) task: Pin<Box<T>>,
    priority: usize,
    state: State,
}

#[derive(Debug)]
pub(super) struct Scheduler<T: ?Sized> {
    list: LinkedList<Entry<T>>,
    id: usize,
}

impl<T: ?Sized> Scheduler<T> {
    pub(super) fn new() -> Self {
        Self {
            list: LinkedList::new(),
            id: 0,
        }
    }

    pub(super) fn current(&self) -> Option<&Entry<T>> {
        self.list.iter().find(|entry| entry.state == State::Waiting)
    }

    pub(super) fn current_id(&self) -> Option<usize> {
        self.list
            .iter()
            .find(|entry| entry.state == State::Waiting)
            .and_then(|e| Some(e.id))
    }

    pub(super) fn current_mut(&mut self) -> Option<&mut Entry<T>> {
        self.list
            .iter_mut()
            .find(|entry| entry.state == State::Waiting)
    }

    pub(super) fn pop(&mut self) -> Option<(Entry<T>, usize)> {
        let mut cursor = self.list.cursor_front();
        let mut prev_id = 0;
        while let Some(current) = cursor.current()
            && let Blocked(id) = current.state
            && id != prev_id
        {
            prev_id = current.id;
            cursor.move_next();
        }

        let res = if let Some(index) = cursor.index() {
            Some((self.list.remove(index), prev_id))
        } else {
            None
        };

        if let Some(prev) = self.find_mut(prev_id) {
            prev.1.state = State::Waiting;
        }

        res
    }

    fn find_max_priority(&self) -> Option<usize> {
        self.list.iter().map(|e| e.priority).max()
    }

    pub(super) fn find(&self, id: usize) -> Option<&Entry<T>> {
        self.list.iter().find(|entry| entry.id == id)
    }

    pub(super) fn find_mut(&mut self, id: usize) -> Option<(usize, &mut Entry<T>)> {
        self.list
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| entry.id == id)
    }

    /// Add a task to scheduler list.
    /// Lower priority places the task closer to the front
    /// of the list.
    pub(super) fn add(&mut self, task: Pin<Box<T>>) -> usize {
        let id = self.id;
        let priority = self
            .find_max_priority()
            .and_then(|p| Some(p + 1))
            .unwrap_or(1);
        let mut cursor = self.list.cursor_front_mut();

        while let Some(current) = cursor.current()
            && current.priority < priority
        {
            cursor.move_next();
        }
        cursor.insert_before(Entry {
            id: id,
            task,
            priority,
            state: State::Waiting,
        });
        self.id = self.id.wrapping_add(1);
        id
    }

    pub(super) fn add_blocking(
        &mut self,
        task: Pin<Box<T>>,
        to_block_id: usize,
    ) -> Result<usize, ()> {
        let id = self.id;
        let to_block_entry_index = if let Some((index, tbe)) = self.find_mut(to_block_id)
            && tbe.state == State::Waiting
        {
            tbe.state = Blocked(id);
            Some((index, tbe.priority))
        } else {
            None
        };

        if let Some((tbei, priority)) = to_block_entry_index {
            let mut tail = self.list.split_off(tbei + 1);
            self.list.push_back(Entry {
                id,
                task,
                priority,
                state: State::Waiting,
            });
            self.list.append(&mut tail);
            self.id = self.id.wrapping_add(1);
            Ok(id)
        } else {
            if let Some(to_block) = self.list.iter_mut().find(|e| e.id == to_block_id) {
                to_block.state = State::Waiting;
            }
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scheduler::Entry;
    use crate::scheduler::Scheduler;
    use crate::scheduler::State;
    use crate::task::Task;

    #[test]
    fn empty() {
        let mut scheduler: Scheduler<usize> = Scheduler::new();
        assert!(scheduler.current().is_none());
        assert!(scheduler.pop().is_none());
    }

    #[test]
    fn one() {
        let mut scheduler = Scheduler::new();
        scheduler.add(Box::pin(0));
        assert!(scheduler.current().is_some());
        assert!(scheduler.pop().is_some());
    }

    #[test]
    fn two() {
        let mut scheduler = Scheduler::new();
        println!("{:?}", scheduler.list);
        let id0 = scheduler.add(Box::pin(0));
        println!("{:?}", scheduler.list);
        let id1 = scheduler.add_blocking(Box::pin(1), id0).unwrap();
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.find(id0).unwrap().state, State::Blocked(id1));
        assert_eq!(scheduler.find(id0).unwrap().priority, 1);
        assert_eq!(scheduler.find(id1).unwrap().state, State::Waiting);
        assert_eq!(scheduler.find(id1).unwrap().priority, 1);
    }

    #[test]
    fn three() {
        let mut scheduler = Scheduler::new();
        println!("{:?}", scheduler.list);
        let id0 = scheduler.add(Box::pin(0));
        println!("{:?}", scheduler.list);
        let id1 = scheduler.add_blocking(Box::pin(1), id0).unwrap();
        println!("{:?}", scheduler.list);
        let id2 = scheduler.add_blocking(Box::pin(2), id1).unwrap();
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.find(id0).unwrap().state, State::Blocked(id1));
        assert_eq!(scheduler.find(id0).unwrap().priority, 1);
        assert_eq!(scheduler.find(id1).unwrap().state, State::Blocked(id2));
        assert_eq!(scheduler.find(id1).unwrap().priority, 1);
        assert_eq!(scheduler.find(id2).unwrap().state, State::Waiting);
        assert_eq!(scheduler.find(id2).unwrap().priority, 1);
    }

    #[test]
    fn current() {
        let mut scheduler = Scheduler::new();
        println!("{:?}", scheduler.list);
        let id0 = scheduler.add(Box::pin(0));
        println!("{:?}", scheduler.list);
        let id1 = scheduler
            .add_blocking(Box::pin(1), scheduler.current_id().unwrap())
            .unwrap();
        println!("{:?}", scheduler.list);
        let id2 = scheduler.add_blocking(Box::pin(2), 1).unwrap();
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.current().unwrap().id, id2);
        assert_eq!(scheduler.current().unwrap().state, State::Waiting);
        assert_eq!(scheduler.current().unwrap().priority, 1);
        let id3 = scheduler.add(Box::pin(3));
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.current().unwrap().state, State::Waiting);
        assert_eq!(scheduler.current().unwrap().priority, 1);
        assert_eq!(scheduler.current().unwrap().id, id2);
    }

    #[test]
    fn pop() {
        let mut scheduler = Scheduler::new();
        println!("{:?}", scheduler.list);
        let id0 = scheduler.add(Box::pin(0));
        println!("{:?}", scheduler.list);
        let id1 = scheduler.add_blocking(Box::pin(1), 0).unwrap();
        println!("{:?}", scheduler.list);
        let id2 = scheduler.add_blocking(Box::pin(2), 1).unwrap();
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.current().unwrap().id, id2);
        assert_eq!(scheduler.current().unwrap().state, State::Waiting);
        assert_eq!(scheduler.current().unwrap().priority, 1);
        let id3 = scheduler.add(Box::pin(3));
        println!("{:?}", scheduler.list);
        assert_eq!(scheduler.current().unwrap().state, State::Waiting);
        assert_eq!(scheduler.current().unwrap().priority, 1);
        assert_eq!(scheduler.current().unwrap().id, id2);
        let popped = scheduler.pop().unwrap();
        println!("{:?}", scheduler.list);
        assert_eq!(popped.0.id, id2);
        assert_eq!(popped.1, id1);
    }
}
