use candid::{Deserialize, CandidType};
use ic_canister::{PreUpdate, Canister, canister_call};
use ic_exports::ic_cdk::export::candid::{Principal};
use test_canister_b::CanisterB;
use ic_tx::{db::IcTx, backend::hashmap::HashmapBackend, model::{Model, NewModel}};
use std::{rc::Rc};

use ic_canister::{query, update};

pub type DbType = IcTx<Data, HashmapBackend::<u32, Data>>;

#[derive(Clone, CandidType, Deserialize)]
pub struct Data {
    username: String,
    tokens: u32
}

thread_local! {
    pub static DB: DbType = IcTx::new(Rc::new(HashmapBackend::new()));
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
        // Withput opening a transaction, you can only read from the db
        // Data is never locked; all reads and writes are executed in parallel
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

        // Rollbacks, the changes are not persisted
        tx.rollback();
    }

    #[update]
    fn update_user(&self, id: u32, tokens: u32) {
        self.update_user_inner(id, tokens)
    }

    fn update_user_inner(&self, id: u32, tokens: u32) {
        // Starts a transation
        let mut tx = self.db().tx();
        
        // Fetches the user data
        let mut user= tx.fetch_one(&id).unwrap();
        user.data.tokens = tokens;

        // Updates the user data
        tx.update(user).unwrap();

        // Commits so the changes to the user data are persisted
        tx.commit();
    }

    #[update]
    async fn update_user_concurrent_error(&self, id: u32, tokens: u32) {

        // Starts a transaction
        let mut tx = self.db().tx();
        
        // Reads user data from the store
        let mut user= tx.fetch_one(&id).unwrap();
        user.data.tokens = tokens * 10;

        // Update the user data
        tx.update(user).unwrap();

        // Calls another canister. Here the IC context is switched and the
        // state should be persisted. Nevertheless, the transaction data is
        // not persisted until commit is excuted
        {
            use test_canister_b::CanisterBImpl;
            let canister_b = CanisterBImpl::from_principal(self.principal);
            canister_call!(canister_b.get_counter(), u32)
            .await
            .unwrap();
        }

        // Here we simulate a concurrent modification of the user data
        self.update_user_inner(id, tokens);

        // We finally commit the transaction.
        // The commit will panic because another call has modified the user data concurrently.
        // Please note that the commit fails because the other call has modified the data of the very same user,
        // if another user was changed, the commit would have succeeded.
        // The state is reverted to its initial value. Even changes performed before the .await call 
        // are reverted so there is no dirty state.
        tx.commit();
    }

}

pub fn generate_idl() -> String {
    use ic_canister::{generate_idl, Idl};
    let canister_idl = generate_idl!();
    candid::bindings::candid::compile(&canister_idl.env.env, &Some(canister_idl.actor))
}
