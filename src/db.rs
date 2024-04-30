use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    OnceLock, RwLock,
};

static ID_MANAGER: OnceLock<IdManager> = OnceLock::new();

struct IdManager {
    current_id: AtomicUsize,
    ids: RwLock<HashMap<Id, AtomicUsize>>,
}

impl Default for IdManager {
    fn default() -> Self {
        Self {
            current_id: AtomicUsize::new(0),
            ids: RwLock::new(HashMap::default()),
        }
    }
}

impl IdManager {
    pub fn init() {
        ID_MANAGER.get_or_init(|| Self::default());
    }
    pub fn new_entry() -> Id {
        let mgr = ID_MANAGER.get().unwrap();
        let id = Id(mgr.current_id.fetch_add(1, Ordering::Relaxed));
        mgr.ids.write().unwrap().insert(id, AtomicUsize::new(0));
        id
    }
    pub fn next_id(id: Id) -> Id {
        let mgr = ID_MANAGER.get().unwrap();
        let ids_write = mgr.ids.write().unwrap();
        let id = ids_write.get(&id).unwrap();
        Id(id.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct DB<T> {
    pub data: HashMap<Id, T>,
    id: Id,
}

impl<T> Default for DB<T> {
    fn default() -> Self {
        IdManager::init();
        let id = IdManager::new_entry();
        Self {
            data: HashMap::new(),
            id,
        }
    }
}

impl<T> DB<T> {
    pub fn insert(&mut self, val: T) -> Id {
        let id = IdManager::next_id(self.id);
        self.data.insert(id, val);
        id
    }

    pub fn get<'a>(&'a self, id: Id) -> &'a T {
        let item = self.data.get(&id);
        item.unwrap()
    }

    pub fn get_mut<'a>(&'a mut self, id: Id) -> &'a mut T {
        let item = self.data.get_mut(&id);
        item.unwrap()
    }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db() {
        let mut db = DB::default();
        let id1 = db.insert(5);
        let id2 = db.insert(10);

        assert!(id1 != id2);
    }
}
