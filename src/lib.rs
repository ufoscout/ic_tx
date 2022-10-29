use std::rc::Rc;

pub mod backend;
pub mod db;
pub mod error;
pub mod model;
pub mod tx;

pub type Ref<T> = Rc<T>;