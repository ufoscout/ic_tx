use std::marker::PhantomData;

use crate::{backend::Backend, tx::Tx, Ref};

pub struct TxMx<Data, B: Backend<Data>> {
    backend: Ref<B>,
    phantom_data: PhantomData<Data>,
}


impl <Data, B: Backend<Data>> TxMx<Data, B> {

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

}