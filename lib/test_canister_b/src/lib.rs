use ic_canister::PreUpdate;
use ic_exports::ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_storage::{stable::Versioned, IcStorage};
use std::{cell::RefCell, rc::Rc};

use ic_canister::{generate_exports, query, state_getter, Canister};

#[derive(Default, CandidType, Deserialize, IcStorage)]
pub struct StateB {
    counter: u32,
}

impl Versioned for StateB {
    type Previous = ();

    fn upgrade((): ()) -> Self {
        Self::default()
    }
}

pub trait CanisterB: Canister {
    #[state_getter]
    fn state(&self) -> Rc<RefCell<StateB>>;

    #[query(trait = true)]
    fn get_counter(&self) -> u32 {
        self.state().borrow().counter
    }
}

generate_exports!(CanisterB, CanisterBImpl);

pub fn generate_idl() -> String {
    use ic_canister::{generate_idl, Idl};
    let canister_idl = generate_idl!();
    candid::bindings::candid::compile(&canister_idl.env.env, &Some(canister_idl.actor))
}
