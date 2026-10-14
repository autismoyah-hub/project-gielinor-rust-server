//! EngineQueueList/EngineQueueCache semantics, separate from delayed script queues.
//! Publication selects labelled handlers before defaults; callers own handlers.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Kind {
    ChangeStat = 1,
    AdvanceStat = 2,
    Mapzone = 3,
    MapzoneExit = 4,
    Zone = 5,
    ZoneExit = 6,
    Area = 7,
    AreaExit = 8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queue<A> {
    pub kind: Kind,
    pub label: i32,
    pub args: A,
}
#[derive(Default)]
pub struct Bindings {
    labelled: BTreeSet<i64>,
    defaults: BTreeSet<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Labelled,
    Default,
}
impl Bindings {
    fn key(kind: Kind, label: i32) -> i64 {
        ((kind as i64) << 32) | i64::from(label)
    }
    pub fn add_labelled(&mut self, kind: Kind, label: i32) {
        self.labelled.insert(Self::key(kind, label));
    }
    pub fn add_default(&mut self, kind: Kind) {
        self.defaults.insert(kind as i32);
    }
    pub fn select(&self, kind: Kind, label: i32) -> Option<Binding> {
        if self.labelled.contains(&Self::key(kind, label)) {
            Some(Binding::Labelled)
        } else if self.defaults.contains(&(kind as i32)) {
            Some(Binding::Default)
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Capacity,
    IdentityExhausted,
    EventBudget,
    Reentrant,
}
struct Node<A> {
    queue: Queue<A>,
    prev: Option<u64>,
    next: Option<u64>,
}
pub struct Queues<A> {
    nodes: BTreeMap<u64, Node<A>>,
    head: Option<u64>,
    tail: Option<u64>,
    next_id: u64,
    processing: bool,
    cleared: bool,
}
impl<A> Default for Queues<A> {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            head: None,
            tail: None,
            next_id: 0,
            processing: false,
            cleared: false,
        }
    }
}
impl<A> Queues<A> {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Admit a related group atomically so callers can commit state only after
    /// all corresponding notifications fit. The bounds are host safeguards.
    pub fn push_batch(&mut self, queues: Vec<Queue<A>>) -> Result<(), Error> {
        if queues.len() > 1024usize.saturating_sub(self.len()) {
            return Err(Error::Capacity);
        }
        self.next_id
            .checked_add(queues.len() as u64)
            .ok_or(Error::IdentityExhausted)?;
        for queue in queues {
            self.push(queue)?;
        }
        Ok(())
    }
    pub fn push(&mut self, queue: Queue<A>) -> Result<(), Error> {
        if self.len() >= 1024 {
            return Err(Error::Capacity);
        }
        let id = self
            .next_id
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        self.next_id = id;
        let node = Node {
            queue,
            prev: self.tail,
            next: None,
        };
        if let Some(tail) = self.tail {
            self.nodes.get_mut(&tail).unwrap().next = Some(id);
        } else {
            self.head = Some(id);
        }
        self.nodes.insert(id, node);
        self.tail = Some(id);
        Ok(())
    }
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        // Kotlin clear() calls the active iterator's cleanUp(). Appending after
        // clear inside a callback must wait until a subsequent processing pass.
        self.cleared = true;
    }
    pub fn iter(&self) -> impl Iterator<Item = &Queue<A>> {
        let mut next = self.head;
        std::iter::from_fn(move || {
            let node = self.nodes.get(&next?)?;
            next = node.next;
            Some(&node.queue)
        })
    }
    /// Callback returns whether access became protected. Protected entries stay
    /// queued. Append-to-tail visibility follows the reference iterator cursor.
    pub fn process(
        &mut self,
        protected: bool,
        mut publish: impl FnMut(&mut Self, Queue<A>) -> bool,
    ) -> Result<(), Error> {
        if self.processing {
            return Err(Error::Reentrant);
        }
        if protected {
            return Ok(());
        }
        self.processing = true;
        self.cleared = false;
        let mut next = self.head;
        let mut count = 0;
        while let Some(id) = next {
            if count >= 4096 {
                self.processing = false;
                return Err(Error::EventBudget);
            }
            count += 1;
            let node = self.nodes.remove(&id).unwrap();
            // Capture next before callback. Appending while processing the old
            // tail is deferred; appending earlier in the pass can be observed.
            next = node.next;
            if let Some(prev) = node.prev {
                self.nodes.get_mut(&prev).unwrap().next = node.next;
            } else {
                self.head = node.next;
            }
            if let Some(after) = node.next {
                self.nodes.get_mut(&after).unwrap().prev = node.prev;
            } else {
                self.tail = node.prev;
            }
            let protected = publish(self, node.queue);
            if protected || self.cleared {
                break;
            }
        }
        self.processing = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(args: i32) -> Queue<i32> {
        Queue {
            kind: Kind::ChangeStat,
            label: 0,
            args,
        }
    }
    #[test]
    fn append_visibility_depends_on_cursor_and_protection_retains_work() {
        for (initial, expected) in [(vec![1], vec![1]), (vec![1, 2], vec![1, 2, 3])] {
            let mut queues = Queues::default();
            for value in initial {
                queues.push(item(value)).unwrap();
            }
            queues
                .process(true, |_, _| panic!("protected callback"))
                .unwrap();
            let mut observed = vec![];
            queues
                .process(false, |queues, event| {
                    observed.push(event.args);
                    if event.args == 1 {
                        queues.push(item(3)).unwrap();
                    }
                    false
                })
                .unwrap();
            assert_eq!(observed, expected);
        }
        let mut queues = Queues::default();
        queues.push(item(1)).unwrap();
        queues.push(item(2)).unwrap();
        queues.process(false, |_, _| true).unwrap();
        assert_eq!(queues.iter().map(|q| q.args).collect::<Vec<_>>(), vec![2]);
    }
    #[test]
    fn clear_then_append_finishes_current_pass_and_bindings_preserve_signed_keys() {
        let mut queues = Queues::default();
        queues.push(item(1)).unwrap();
        queues.push(item(2)).unwrap();
        queues
            .process(false, |queues, _| {
                queues.clear();
                queues.push(item(3)).unwrap();
                false
            })
            .unwrap();
        assert_eq!(queues.iter().map(|q| q.args).collect::<Vec<_>>(), vec![3]);
        let mut bindings = Bindings::default();
        bindings.add_default(Kind::AdvanceStat);
        bindings.add_labelled(Kind::AdvanceStat, 3);
        assert_eq!(
            bindings.select(Kind::AdvanceStat, 3),
            Some(Binding::Labelled)
        );
        assert_eq!(
            bindings.select(Kind::AdvanceStat, 4),
            Some(Binding::Default)
        );
        // The reference ORs a sign-extended label; negative labels alias kinds.
        bindings.add_labelled(Kind::ChangeStat, -1);
        assert_eq!(bindings.select(Kind::AreaExit, -1), Some(Binding::Labelled));
    }
}
