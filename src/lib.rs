use std::rc::Rc;

mod action;
pub mod backend;
pub mod db;
pub mod model;
pub mod tx;

pub type Ref<T> = Rc<T>;