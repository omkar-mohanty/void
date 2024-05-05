use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct DB<T> {
    pub data: HashMap<Id, T>,
    next_id: AtomicUsize,
}

impl<T> Default for DB<T> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            next_id: AtomicUsize::new(0),
        }
    }
}

impl<T> DB<T> {
    pub fn insert(&mut self, val: T) -> Id {
        let id = Id(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.data.insert(id, val);
        id
    }

    pub fn get<'a>(&'a self, id: Id) -> &'a T {
        let item = self.data.get(&id);
        item.unwrap()
    }

    pub fn get_all<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.data.values()
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
