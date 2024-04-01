mod api;
mod model;
mod texture;

use std::collections::HashMap;

pub use api::*;
pub use model::*;
pub use texture::{TextureDesc, TextureId};

use void_core::db::{IDb, IId};

pub struct ResourceDB<I: IId, T> {
    resources: HashMap<I, T>,
}

impl<I: IId, T> IDb for ResourceDB<I, T> {
    type Data = T;
    type Id = I;
    fn get(
        &self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> Result<impl Iterator<Item = &Self::Data>, void_core::db::DbError<Self::Id>> {
        let filtered_resource = ids.filter_map(|id| self.resources.get(&id));
        Ok(filtered_resource)
    }
    fn get_by_id(&self, id: &Self::Id) -> Option<&Self::Data> {
        self.resources.get(id)
    }
    fn iter(&self) -> impl Iterator<Item = (&Self::Id, &Self::Data)> {
        self.resources.iter()
    }
}
