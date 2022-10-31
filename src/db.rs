use std::marker::PhantomData;

use crate::{backend::Backend, tx::Tx, Ref, model::Model, error::TxError};

pub struct IcTx<Data, B: Backend<Data>> {
    backend: Ref<B>,
    phantom_data: PhantomData<Data>,
}

impl <Data, B: Backend<Data>> Clone for IcTx<Data, B> {
    fn clone(&self) -> Self {
        Self { backend: self.backend.clone(), phantom_data: PhantomData }
    }
}

impl <Data, B: Backend<Data>> IcTx<Data, B> {

    pub fn new(backend: Ref<B>) -> Self {
        Self { 
            backend, 
            phantom_data: PhantomData,
        }
    }

    /// Starts a new atomic transaction
    pub fn tx(&self) -> Tx<Data, B> {
        Tx::new(self.backend.clone())
    }

    pub fn fetch_one(&self, id: &B::IdType) -> Result<Model<B::IdType, Data>, TxError> {
        self.backend.fetch_one(id)
    }

    pub fn fetch_option_one(&self, id: &B::IdType) -> Result<Option<Model<B::IdType, Data>>, TxError> {
        self.backend.fetch_option_one(id)
    }

}