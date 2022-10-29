use crate::{model::{Model, NewModel, VersionType}};

pub mod hashmap;

pub trait Backend<Data> {

    type IdType;
    type Error;

    fn fetch_one(&self, id: &Self::IdType) -> Result<Model<Self::IdType, Data>, Self::Error>;
    fn fetch_option_one(&self, id: &Self::IdType) -> Result<Option<Model<Self::IdType, Data>>, Self::Error>;
    fn fetch_version(&self, id: &Self::IdType) -> Result<VersionType, Self::Error>;
    fn fetch_option_version(&self, id: &Self::IdType) -> Result<Option<VersionType>, Self::Error>;
    fn update(&self, model: Model<Self::IdType, Data>) -> Result<(), Self::Error>;
    fn delete(&self, id: &Self::IdType) -> Result<(), Self::Error>;
    fn delete_option(&self, id: &Self::IdType) -> Result<bool, Self::Error>;
    fn save(&self, model: NewModel<Self::IdType, Data>) -> Result<(), Self::Error>;
    
}