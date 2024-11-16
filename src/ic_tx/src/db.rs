use std::{cell::RefCell, marker::PhantomData};

use crate::{backend::Backend, error::TxError, model::Model, tx::Tx, Ref};

pub struct IcTx<Data, B: Backend<Data>> {
    backend: Ref<RefCell<B>>,
    phantom_data: PhantomData<Data>,
}

impl<Data, B: Backend<Data>> Clone for IcTx<Data, B> {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<Data, B: Backend<Data>> IcTx<Data, B> {
    pub fn new(backend: Ref<RefCell<B>>) -> Self {
        Self {
            backend,
            phantom_data: PhantomData,
        }
    }

    /// Starts a new atomic transaction
    pub fn tx(&self) -> Tx<Data, B> {
        Tx::new(self.backend.clone())
    }

    /// Fetches a model from the database.
    /// Returns an error if no model is found with the specified id.
    pub fn fetch_one(&self, id: &B::IdType) -> Result<Model<B::IdType, Data>, TxError> {
        self.backend.borrow().fetch_one(id)
    }

    /// Fetches a model from the database.
    pub fn fetch_option_one(
        &self,
        id: &B::IdType,
    ) -> Result<Option<Model<B::IdType, Data>>, TxError> {
        self.backend.borrow().fetch_option_one(id)
    }
}
