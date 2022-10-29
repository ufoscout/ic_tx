use std::fmt::Display;

use crate::{model::{Model, NewModel, VersionType}, error::TxError};

pub mod hashmap;

pub trait Backend<Data> {

    type IdType: Display + Clone;

    fn fetch_one(&self, id: &Self::IdType) -> Result<Model<Self::IdType, Data>, TxError>;
    fn fetch_option_one(&self, id: &Self::IdType) -> Result<Option<Model<Self::IdType, Data>>, TxError>;
    fn fetch_version(&self, id: &Self::IdType) -> Result<VersionType, TxError>;
    fn fetch_option_version(&self, id: &Self::IdType) -> Result<Option<VersionType>, TxError>;
    fn update(&self, model: Model<Self::IdType, Data>) -> Result<(), TxError>;
    fn delete(&self, id: &Self::IdType) -> Result<(), TxError>;
    fn delete_option(&self, id: &Self::IdType) -> Result<bool, TxError>;
    fn save(&self, model: NewModel<Self::IdType, Data>) -> Result<(), TxError>;
    
}