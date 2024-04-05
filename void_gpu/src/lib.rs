pub mod api;
pub mod io;
pub mod model;
pub mod texture;

use std::collections::HashMap;
use std::sync::RwLock;
use void_core::db::{IDb, IId};
use void_core::threadpool::rayon::iter::IntoParallelIterator;

pub struct ResourceDB<I: IId, T> {
    resources: HashMap<I, T>,
}

impl<I: IId, T> Default for ResourceDB<I, T> {
    fn default() -> Self {
        ResourceDB {
            resources: HashMap::new(),
        }
    }
}

impl<I: IId, T> IDb for ResourceDB<I, T> {
    type Data = T;
    type Id = I;
    fn get_by_id(&self, id: &Self::Id) -> Option<&Self::Data> {
        self.resources.get(id)
    }
    fn iter(&self) -> impl Iterator<Item = (&Self::Id, &Self::Data)> {
        self.resources.iter()
    }
    fn values(&self) -> impl Iterator<Item = &Self::Data> {
        self.resources.values()
    }
    fn keys(&self) -> impl Iterator<Item = &Self::Id> {
        self.resources.keys()
    }
}
