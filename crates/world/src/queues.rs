//! PlayerQueueProcessor's mutation-sensitive iteration and category rules.
//! Content callbacks run synchronously; their suspension state is returned through Access.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Category {
    Soft = 0,
    Normal = 1,
    Weak = 2,
    Strong = 3,
    LongAccelerate = 4,
    LongDiscard = 5,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queue<A> {
    pub id: i32,
    pub category: Category,
    pub remaining: i32,
    pub processed_cycle: i32,
    pub args: A,
}
struct Node<A> {
    queue: Queue<A>,
    prev: Option<u64>,
    next: Option<u64>,
    active: bool,
}
struct List<A> {
    nodes: BTreeMap<u64, Node<A>>,
    head: Option<u64>,
    tail: Option<u64>,
    cursor: Option<u64>,
    next_id: u64,
    size: usize,
}
impl<A> Default for List<A> {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            head: None,
            tail: None,
            cursor: None,
            next_id: 0,
            size: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Capacity,
    IdentityExhausted,
    IteratorInvalidated,
    EventBudget,
}
impl<A> List<A> {
    fn add(&mut self, queue: Queue<A>) -> Result<(), Error> {
        if self.size >= 1024 {
            return Err(Error::Capacity);
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let id = self.next_id;
        self.nodes.insert(
            id,
            Node {
                queue,
                prev: self.tail,
                next: None,
                active: true,
            },
        );
        if let Some(tail) = self.tail {
            self.nodes.get_mut(&tail).unwrap().next = Some(id);
        } else {
            self.head = Some(id);
        }
        self.tail = Some(id);
        self.size += 1;
        Ok(())
    }
    fn remove(&mut self, id: u64) -> Result<(), Error> {
        let node = self.nodes.get_mut(&id).ok_or(Error::IteratorInvalidated)?;
        if !node.active {
            return Err(Error::IteratorInvalidated);
        }
        node.active = false;
        let (prev, next) = (node.prev, node.next);
        if let Some(prev) = prev {
            self.nodes.get_mut(&prev).unwrap().next = next;
        } else {
            self.head = next;
        }
        if let Some(next) = next {
            self.nodes.get_mut(&next).unwrap().prev = prev;
        } else {
            self.tail = prev;
        }
        self.size -= 1;
        Ok(())
    }
    fn remove_type(&mut self, kind: i32) -> usize {
        let ids: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.active && n.queue.id == kind)
            .map(|(&id, _)| id)
            .collect();
        for &id in &ids {
            self.remove(id).unwrap();
        }
        ids.len()
    }
    fn clear(&mut self) {
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        self.cursor = None;
        self.size = 0;
    }
    fn cleanup(&mut self) {
        self.cursor = None;
        self.nodes.retain(|_, n| n.active);
    }
}
#[derive(Default, Debug)]
pub struct Access {
    pub protected: bool,
    pub pending_logout: bool,
}
#[derive(Clone, Debug)]
pub enum Event<A> {
    CloseModals,
    Launch(Queue<A>),
}
pub struct Scheduler<A> {
    regular: List<A>,
    weak: List<A>,
}
impl<A> Default for Scheduler<A> {
    fn default() -> Self {
        Self {
            regular: List::default(),
            weak: List::default(),
        }
    }
}
impl<A: Clone> Scheduler<A> {
    pub fn add(
        &mut self,
        id: i32,
        category: Category,
        remaining: i32,
        args: A,
    ) -> Result<(), Error> {
        let queue = Queue {
            id,
            category,
            remaining,
            processed_cycle: -1,
            args,
        };
        if category == Category::Weak {
            self.weak.add(queue)
        } else {
            self.regular.add(queue)
        }
    }
    pub fn remove_all(&mut self, id: i32) -> usize {
        let removed = self.regular.remove_type(id) + self.weak.remove_type(id);
        if self.regular.cursor.is_none() {
            self.regular.cleanup();
        }
        if self.weak.cursor.is_none() {
            self.weak.cleanup();
        }
        removed
    }
    pub fn clear(&mut self) {
        self.regular.clear();
        self.weak.clear();
    }
    pub fn clear_weak(&mut self) {
        self.weak.clear();
    }
    pub fn len(&self) -> usize {
        self.regular.size + self.weak.size
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// PlayerLogoutProcess scans the regular list only. Weak queues are not a
    /// save barrier; LongDiscard is the sole discardable regular category.
    pub fn has_non_discardable(&self) -> bool {
        self.regular
            .nodes
            .values()
            .any(|node| node.active && node.queue.category != Category::LongDiscard)
    }
    pub fn contains(&self, id: i32) -> bool {
        self.snapshot().iter().any(|q| q.id == id)
    }
    pub fn snapshot(&self) -> Vec<Queue<A>> {
        [&self.regular, &self.weak]
            .into_iter()
            .flat_map(|l| {
                l.nodes
                    .values()
                    .filter(|n| n.active)
                    .map(|n| n.queue.clone())
            })
            .collect()
    }
    fn list(&mut self, weak: bool) -> &mut List<A> {
        if weak {
            &mut self.weak
        } else {
            &mut self.regular
        }
    }
    /// A budget makes a content callback which endlessly requeues itself observable instead
    /// of hanging the world. Once-per-cycle decrement and unprocessed queues are retained.
    pub fn process(
        &mut self,
        clock: i32,
        access: &mut Access,
        budget: usize,
        mut publish: impl FnMut(&mut Self, &mut Access, Event<A>),
    ) -> Result<(), Error> {
        let mut emitted = 0;
        let result = (|| {
            if self
                .regular
                .nodes
                .values()
                .any(|n| n.active && n.queue.category == Category::Strong)
            {
                if emitted >= budget {
                    return Err(Error::EventBudget);
                }
                emitted += 1;
                publish(self, access, Event::CloseModals);
            }
            for weak in [false, true] {
                while self.list(weak).size > 0 {
                    let mut processed_none = true;
                    self.list(weak).cursor = self.list(weak).head;
                    while let Some(id) = self.list(weak).cursor {
                        // Capture the next link before callbacks; appended nodes are seen only
                        // if a later node's next() runs, exactly like the Kotlin linked iterator.
                        let node = self
                            .list(weak)
                            .nodes
                            .get(&id)
                            .ok_or(Error::IteratorInvalidated)?;
                        let (next, category) = (node.next, node.queue.category);
                        self.list(weak).cursor = next;
                        if !weak && matches!(category, Category::Strong | Category::Soft) {
                            if emitted >= budget {
                                return Err(Error::EventBudget);
                            }
                            emitted += 1;
                            publish(self, access, Event::CloseModals);
                        }
                        // ifClose may clear the list; the source still visits the detached
                        // queue. Report invalidation instead of fabricating a new live queue.
                        let node = self
                            .list(weak)
                            .nodes
                            .get_mut(&id)
                            .ok_or(Error::IteratorInvalidated)?;
                        if node.queue.processed_cycle != clock {
                            node.queue.processed_cycle = clock;
                            node.queue.remaining = node.queue.remaining.wrapping_sub(1);
                        }
                        if !weak && access.pending_logout && category == Category::LongAccelerate {
                            node.queue.remaining = 0;
                        }
                        if node.queue.remaining > 0
                            || (category != Category::Soft && access.protected)
                        {
                            continue;
                        }
                        if emitted >= budget {
                            return Err(Error::EventBudget);
                        }
                        let queue = node.queue.clone();
                        self.list(weak).remove(id)?;
                        processed_none = false;
                        emitted += 1;
                        publish(self, access, Event::Launch(queue));
                    }
                    self.list(weak).cleanup();
                    // The weak processor also checks the REGULAR list's size in Kotlin.
                    if processed_none || self.regular.size == 1 {
                        break;
                    }
                }
            }
            Ok(())
        })();
        self.regular.cleanup();
        self.weak.cleanup();
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn death_queue_speedup_depends_on_whether_another_node_remains() {
        for (count, expected) in [(1, vec![1]), (2, vec![1, 2, 99])] {
            let mut s = Scheduler::default();
            for id in 1..=count {
                s.add(id, Category::Normal, 1, ()).unwrap();
            }
            let mut fired = vec![];
            s.process(1, &mut Access::default(), 20, |s, _, event| {
                if let Event::Launch(q) = event {
                    fired.push(q.id);
                    if q.id == 1 {
                        s.add(99, Category::Normal, 1, ()).unwrap();
                    }
                }
            })
            .unwrap();
            assert_eq!(fired, expected);
        }
    }
    #[test]
    fn soft_bypasses_protection_strong_closes_modals_and_logout_accelerates() {
        let mut s = Scheduler::default();
        s.add(1, Category::Strong, 2, ()).unwrap();
        s.add(2, Category::Soft, 1, ()).unwrap();
        s.add(3, Category::LongAccelerate, 100, ()).unwrap();
        let mut access = Access {
            protected: true,
            pending_logout: true,
        };
        let mut fired = vec![];
        let mut closed = 0;
        s.process(5, &mut access, 20, |_, _, e| match e {
            Event::CloseModals => closed += 1,
            Event::Launch(q) => fired.push(q.id),
        })
        .unwrap();
        assert_eq!(fired, [2]);
        assert!(closed >= 3);
        assert_eq!(
            s.snapshot().iter().map(|q| q.remaining).collect::<Vec<_>>(),
            [1, 0]
        );
        access.protected = false;
        s.process(5, &mut access, 20, |_, _, e| {
            if let Event::Launch(q) = e {
                fired.push(q.id)
            }
        })
        .unwrap();
        assert_eq!(fired, [2, 3]);
        s.process(6, &mut access, 20, |_, _, e| {
            if let Event::Launch(q) = e {
                fired.push(q.id)
            }
        })
        .unwrap();
        assert_eq!(fired, [2, 3, 1]);
    }
    #[test]
    fn weak_append_quirk_and_clear_terminate_iterator() {
        let mut s = Scheduler::default();
        s.add(10, Category::Normal, 100, ()).unwrap();
        s.add(1, Category::Weak, 1, ()).unwrap();
        let mut fired = vec![];
        s.process(1, &mut Access::default(), 20, |s, _, e| {
            if let Event::Launch(q) = e {
                fired.push(q.id);
                s.add(2, Category::Weak, 1, ()).unwrap();
            }
        })
        .unwrap();
        assert_eq!(fired, [1]);
        s.process(2, &mut Access::default(), 20, |s, _, e| {
            if let Event::Launch(q) = e {
                fired.push(q.id);
                s.clear();
            }
        })
        .unwrap();
        assert_eq!(fired, [1, 2]);
        assert!(s.is_empty());
    }
}
