use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

pub trait IId: Clone + Copy + Hash + Eq + PartialEq + Debug {}

pub trait IDb {
    type Id: IId;
    type Data;

    fn get(
        &self,
        ids: impl Iterator<Item = Self::Id>,
    ) -> Result<impl Iterator<Item = &Self::Data>, DbError<Self::Id>>;
}

#[derive(Error, Debug)]
pub enum DbError<I: IId> {
    #[error("Invalid ID in input iterator : {0}")]
    InvaidID(I),
}
