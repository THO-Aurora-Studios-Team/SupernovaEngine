//! Component trait and sparse-set storage.

use std::alloc::{self, Layout};
use std::any::TypeId;
use std::ptr::NonNull;

use crate::entity::{Entity, EntityIndex};

/// Marker trait for components. Implement on any `'static + Send + Sync` type.
pub trait Component: 'static + Send + Sync {}

/// Blanket implementation — any `'static + Send + Sync` type is a component.
impl<T: 'static + Send + Sync> Component for T {}

// --------------------------------------------------------------------------- //
// SparseSet — the core component storage
// --------------------------------------------------------------------------- //

/// Type-erased sparse-set storage for a single component type.
///
/// Layout:
/// - `sparse`: entity-index → dense-index (or `EMPTY`)
/// - `dense`:  entity ids, contiguous
/// - `data`:   component values, contiguous, parallel to `dense`
///
/// This gives O(1) lookup by entity and very fast linear iteration.
pub struct ComponentStorage {
    sparse: Vec<u32>,       // indexed by EntityIndex
    dense: Vec<Entity>,     // entities that have this component
    data: NonNull<u8>,      // raw component data buffer
    len: usize,
    cap: usize,
    layout: Layout,
    type_id: TypeId,
    type_name: &'static str,
    drop_fn: unsafe fn(*mut u8, usize),
    // We need to track the size for safety.
    item_size: usize,
}

const EMPTY: u32 = u32::MAX;

impl ComponentStorage {
    /// Create a new sparse-set for component type `T`.
    pub fn new<T: Component>() -> Self {
        let layout = Layout::new::<T>();
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: NonNull::dangling(),
            len: 0,
            cap: 0,
            layout,
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            drop_fn: drop_slice::<T>,
            item_size: std::mem::size_of::<T>(),
        }
    }

    /// Get the TypeId of the stored component.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Ensure capacity for at least `new_cap` elements.
    fn ensure_cap(&mut self, new_cap: usize) {
        if new_cap <= self.cap {
            return;
        }
        let new_cap = new_cap.max(8).max(self.cap * 2);
        unsafe {
            let new_layout = Layout::from_size_align_unchecked(
                new_cap * self.item_size,
                self.layout.align(),
            );
            let new_ptr = if self.cap == 0 {
                alloc::alloc(new_layout)
            } else {
                let old_layout = Layout::from_size_align_unchecked(
                    self.cap * self.item_size,
                    self.layout.align(),
                );
                alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size())
            };
            if new_ptr.is_null() {
                alloc::handle_alloc_error(new_layout);
            }
            self.data = NonNull::new_unchecked(new_ptr);
            self.cap = new_cap;
        }
    }

    /// Ensure the sparse array is large enough for `entity_index`.
    fn ensure_sparse(&mut self, entity_index: EntityIndex) {
        let needed = entity_index as usize + 1;
        if self.sparse.len() < needed {
            self.sparse.resize(needed, EMPTY);
        }
    }

    /// Insert a component for `entity`, overwriting if already present.
    pub fn insert<T: Component>(&mut self, entity: Entity, value: T) {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        self.ensure_sparse(entity.index());
        let sparse_idx = self.sparse[entity.index() as usize];
        if sparse_idx != EMPTY {
            // Overwrite existing
            unsafe {
                let ptr = self.data.as_ptr().add(sparse_idx as usize * self.item_size) as *mut T;
                ptr.write(value);
            }
        } else {
            // New entry
            self.ensure_cap(self.len + 1);
            let dense_idx = self.len;
            unsafe {
                let ptr = self.data.as_ptr().add(dense_idx * self.item_size) as *mut T;
                ptr.write(value);
            }
            self.dense.push(entity);
            self.sparse[entity.index() as usize] = dense_idx as u32;
            self.len += 1;
        }
    }

    /// Remove the component from `entity`. Returns `true` if it existed.
    pub fn remove(&mut self, entity: Entity) -> bool {
        if entity.index() as usize >= self.sparse.len() {
            return false;
        }
        let sparse_idx = self.sparse[entity.index() as usize];
        if sparse_idx == EMPTY {
            return false;
        }
        let dense_idx = sparse_idx as usize;
        let last_idx = self.len - 1;
        unsafe {
            // Drop the removed element
            let ptr = self.data.as_ptr().add(dense_idx * self.item_size);
            (self.drop_fn)(ptr, 1);

            if dense_idx != last_idx {
                // Swap with last
                let last_ptr = self.data.as_ptr().add(last_idx * self.item_size);
                std::ptr::copy_nonoverlapping(last_ptr, ptr, self.item_size);
                let swapped_entity = self.dense[last_idx];
                self.dense[dense_idx] = swapped_entity;
                self.sparse[swapped_entity.index() as usize] = dense_idx as u32;
            }
        }
        self.dense.pop();
        self.sparse[entity.index() as usize] = EMPTY;
        self.len -= 1;
        true
    }

    /// Get a reference to the component for `entity`.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        if entity.index() as usize >= self.sparse.len() {
            return None;
        }
        let sparse_idx = self.sparse[entity.index() as usize];
        if sparse_idx == EMPTY {
            return None;
        }
        unsafe {
            let ptr = self.data.as_ptr().add(sparse_idx as usize * self.item_size) as *const T;
            Some(&*ptr)
        }
    }

    /// Get a mutable reference to the component for `entity`.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        if entity.index() as usize >= self.sparse.len() {
            return None;
        }
        let sparse_idx = self.sparse[entity.index() as usize];
        if sparse_idx == EMPTY {
            return None;
        }
        unsafe {
            let ptr = self.data.as_ptr().add(sparse_idx as usize * self.item_size) as *mut T;
            Some(&mut *ptr)
        }
    }

    /// Returns `true` if `entity` has this component.
    pub fn contains(&self, entity: Entity) -> bool {
        if entity.index() as usize >= self.sparse.len() {
            return false;
        }
        self.sparse[entity.index() as usize] != EMPTY
    }

    /// Get a slice of all component data (typed).
    ///
    /// # Safety
    /// The caller must ensure no mutable borrows are active.
    pub unsafe fn as_slice<T: Component>(&self) -> &[T] {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len)
    }

    /// Get a mutable slice of all component data (typed).
    ///
    /// # Safety
    /// The caller must ensure no other borrows are active.
    pub unsafe fn as_mut_slice<T: Component>(&mut self) -> &mut [T] {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, self.len)
    }

    /// Get all (entity, component) pairs (typed).
    ///
    /// # Safety
    /// The caller must ensure no mutable borrows are active.
    pub unsafe fn entities_and_data<T: Component>(&self) -> (&[Entity], &[T]) {
        debug_assert_eq!(self.type_id, TypeId::of::<T>());
        let data = std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len);
        (&self.dense, data)
    }

    /// Dense entity list.
    pub fn entities(&self) -> &[Entity] {
        &self.dense
    }

    /// Remove all components (used when despawning entities).
    pub fn remove_entity(&mut self, entity: Entity) {
        self.remove(entity);
    }
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        if self.cap > 0 {
            unsafe {
                (self.drop_fn)(self.data.as_ptr(), self.len);
                let layout = Layout::from_size_align_unchecked(
                    self.cap * self.item_size,
                    self.layout.align(),
                );
                alloc::dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

unsafe fn drop_slice<T>(ptr: *mut u8, len: usize) {
    let slice = std::slice::from_raw_parts_mut(ptr as *mut T, len);
    for item in slice {
        std::ptr::drop_in_place(item);
    }
}

// We need Send + Sync because we'll use UnsafeCell-like access patterns
// controlled by the World's borrow checking.
unsafe impl Send for ComponentStorage {}
unsafe impl Sync for ComponentStorage {}
