//! Event system — double-buffered event storage with reader/writer handles.

use std::marker::PhantomData;

/// Trait for event types.
pub trait Event: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> Event for T {}

/// Double-buffered event storage for a single event type.
///
/// Events sent this frame are available to readers. At the end of the frame
/// (or when `flush` is called), the buffer rotates: current events become
/// "previous" and can still be read for one more frame, then they're cleared.
pub struct Events<E: Event> {
    buffer_a: Vec<E>,
    buffer_b: Vec<E>,
    /// Index of the "current" (write) buffer.
    current_is_a: bool,
    /// Number of events in the current buffer at the start of this frame.
    /// Readers created at the same time share this cursor.
    start_count: usize,
}

impl<E: Event> Events<E> {
    pub fn new() -> Self {
        Self {
            buffer_a: Vec::new(),
            buffer_b: Vec::new(),
            current_is_a: true,
            start_count: 0,
        }
    }

    /// Send an event.
    pub fn send(&mut self, event: E) {
        self.current_buffer_mut().push(event);
    }

    fn current_buffer(&self) -> &Vec<E> {
        if self.current_is_a {
            &self.buffer_a
        } else {
            &self.buffer_b
        }
    }

    fn current_buffer_mut(&mut self) -> &mut Vec<E> {
        if self.current_is_a {
            &mut self.buffer_a
        } else {
            &mut self.buffer_b
        }
    }

    fn previous_buffer(&self) -> &Vec<E> {
        if self.current_is_a {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    /// Iterate over events sent this frame.
    pub fn iter_current(&self) -> impl Iterator<Item = &E> {
        self.current_buffer().iter()
    }

    /// Iterate over events from the previous frame.
    pub fn iter_previous(&self) -> impl Iterator<Item = &E> {
        self.previous_buffer().iter()
    }

    /// Number of events this frame.
    pub fn len(&self) -> usize {
        self.current_buffer().len()
    }

    pub fn is_empty(&self) -> bool {
        self.current_buffer().is_empty()
    }

    /// Flush: rotate buffers. Call at end of frame.
    pub fn flush(&mut self) {
        // Clear the "previous" buffer (which will become the new "current")
        if self.current_is_a {
            self.buffer_b.clear();
        } else {
            self.buffer_a.clear();
        }
        self.current_is_a = !self.current_is_a;
        self.start_count = self.current_buffer().len();
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.buffer_a.clear();
        self.buffer_b.clear();
        self.start_count = 0;
    }
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Reader for events of type `E`.
pub struct EventReader<E: Event> {
    cursor: usize,
    _marker: PhantomData<E>,
}

impl<E: Event> EventReader<E> {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            _marker: PhantomData,
        }
    }

    /// Read all unread events from the current buffer.
    pub fn read<'w>(&mut self, events: &'w Events<E>) -> impl Iterator<Item = &'w E> {
        let count = events.len();
        let start = self.cursor.min(count);
        self.cursor = count;
        events.current_buffer()[start..].iter()
    }

    /// Read events from both current and previous frame (useful for
    /// catching events even if a system runs late).
    pub fn read_with_previous<'w>(
        &mut self,
        events: &'w Events<E>,
    ) -> impl Iterator<Item = &'w E> {
        let count = events.len();
        let start = self.cursor.min(count);
        self.cursor = count;
        events.current_buffer()[start..]
            .iter()
            .chain(events.previous_buffer().iter())
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}

impl<E: Event> Default for EventReader<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer for events of type `E`.
pub struct EventWriter<E: Event> {
    _marker: PhantomData<E>,
}

impl<E: Event> EventWriter<E> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Send an event through the world.
    pub fn send(&self, world: &mut crate::world::World, event: E) {
        world.send_event(event);
    }
}

impl<E: Event> Default for EventWriter<E> {
    fn default() -> Self {
        Self::new()
    }
}
