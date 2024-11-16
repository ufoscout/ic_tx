use std::{cell::RefCell, rc::Rc};

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
