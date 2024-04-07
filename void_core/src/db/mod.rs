use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

pub trait IId: Clone + Copy + Hash + Eq + PartialEq + Debug {
    fn new() -> Self;
}

pub trait IDb {
    type Id: IId;
    type Data;

    fn get_by_id(&self, id: &Self::Id) -> Option<&Self::Data>;
    fn iter(&self) -> impl Iterator<Item = (&Self::Id, &Self::Data)>;
    fn keys(&self) -> impl Iterator<Item = &Self::Id>;
    fn values(&self) -> impl Iterator<Item = &Self::Data>;
    fn insert(&mut self, data: impl Iterator<Item = Self::Data>);
}

#[derive(Error, Debug)]
pub enum DbError<I: IId> {
    #[error("Invalid ID in input iterator : {0}")]
    InvaidID(I),
}
