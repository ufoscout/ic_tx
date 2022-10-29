use std::marker::PhantomData;

use crate::{backend::Backend, Ref};

pub struct Tx<Data, B: Backend<Data>> {
    backend: Ref<B>,
    phantom_data: PhantomData<Data>,
}

impl <Data, B: Backend<Data>> Tx<Data, B> {

    pub(crate) fn new(backend: Ref<B>) -> Self {
        Self { 
            backend, 
            phantom_data: PhantomData,
        }
    }

}