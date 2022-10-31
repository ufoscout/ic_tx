use candid::{Deserialize, CandidType};
use ic_canister::{PreUpdate, Canister};
use ic_exports::ic_cdk::export::candid::{Principal};
use tx_mx::{db::TxMx, backend::hashmap::HashmapBackend, model::{Model, NewModel}};
use std::{rc::Rc};

use ic_canister::{query, update};

pub type DbType = TxMx<Data, HashmapBackend::<u32, Data>>;

#[derive(Clone, CandidType, Deserialize)]
pub struct Data {
    username: String,
    tokens: u32
}

thread_local! {
    pub static DB: DbType = TxMx::new(Rc::new(HashmapBackend::new()));
}

#[derive(Canister)]
pub struct CanisterA {
    #[id]
    principal: Principal,
}

impl PreUpdate for CanisterA {}

impl CanisterA {

    fn db(&self) -> DbType {
        DB.with(|c| (*c).clone())
    }

    #[query]
    fn get_user(&self, id: u32) -> Option<Model<u32, Data>> {
        let db = self.db();
        // You can read from the db without opening a transaction
        db.fetch_option_one(&id).unwrap()
    }

    #[update]
    fn create_user(&self, id: u32, username: String) {
        let mut tx = self.db().tx();
        tx.save(NewModel::new(id, Data {
            tokens: 0,
            username
        })).unwrap();

        // Now data is persisted
        tx.commit();
    }

    #[update]
    fn create_user_rollback(&self, id: u32, username: String) {
        let mut tx = self.db().tx();
        tx.save(NewModel::new(id, Data {
            tokens: 0,
            username
        })).unwrap();

        // Data is not persisted
        tx.rollback();
    }

    #[update]
    fn update_user(&self, id: u32, tokens: u32) {
        self.update_user_inner(id, tokens)
    }

    fn update_user_inner(&self, id: u32, tokens: u32) {
        let mut tx = self.db().tx();
        
        let mut user= tx.fetch_one(&id).unwrap();
        user.data.tokens = tokens;

        tx.update(user).unwrap();

        tx.commit();
    }

    #[update]
    fn update_user_concurrent_error(&self, id: u32, tokens: u32) {
        let mut tx = self.db().tx();
        
        let mut user= tx.fetch_one(&id).unwrap();
        user.data.tokens = tokens * 10;
        tx.update(user).unwrap();

        self.update_user_inner(id, tokens);

        // This commit fails because of concurrent modification of the user data
        tx.commit();
    }

}

pub fn generate_idl() -> String {
    use ic_canister::{generate_idl, Idl};
    let canister_idl = generate_idl!();
    candid::bindings::candid::compile(&canister_idl.env.env, &Some(canister_idl.actor))
}
