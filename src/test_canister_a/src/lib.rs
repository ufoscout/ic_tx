use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use ic_tx::{
    backend::hashmap::HashmapBackend,
    db::IcTx,
    model::{Model, NewModel},
};
use std::{rc::Rc, cell::RefCell};

pub type DbType = IcTx<Data, HashmapBackend<u32, Data>>;


thread_local! {
    static CONFIG: RefCell<Config> = RefCell::new(Config::default());
    pub static DB: DbType = IcTx::new(Rc::new(RefCell::new(HashmapBackend::new())));
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
pub struct Data {
    username: String,
    tokens: u32,
}

struct Config {
    pub canister_b_principal: Principal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            canister_b_principal: Principal::anonymous(),
        }
    }
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct InitArgs {
    pub canister_b_principal: Principal,
}

#[ic_cdk::init]
fn init(arg: InitArgs) {
    CONFIG.with(|c| {
        c.replace(Config {
            canister_b_principal: arg.canister_b_principal,
        })
    });
}

    fn db() -> DbType {
        DB.with(|c| (*c).clone())
    }

    #[query]
    fn get_user(id: u32) -> Option<Model<u32, Data>> {
        let db = db();
        // Withput opening a transaction, you can only read from the db
        // Data is never locked; all reads and writes are executed in parallel
        db.fetch_option_one(&id).unwrap()
    }

    #[update]
    fn create_user(id: u32, username: String) {
        let mut tx = db().tx();
        tx.save(NewModel::new(
            id,
            Data {
                tokens: 0,
                username,
            },
        ))
        .unwrap();

        // Now data is persisted
        tx.commit();
    }

    #[update]
    fn create_user_rollback(id: u32, username: String) {
        let mut tx = db().tx();
        tx.save(NewModel::new(
            id,
            Data {
                tokens: 0,
                username,
            },
        ))
        .unwrap();

        // Rollbacks, the changes are not persisted
        tx.rollback();
    }

    #[update]
    fn update_user(id: u32, tokens: u32) {
        update_user_inner(id, tokens)
    }

    fn update_user_inner(id: u32, tokens: u32) {
        // Starts a transation
        let mut tx = 
        db().tx();

        // Fetches the user data
        let mut user = tx.fetch_one(&id).unwrap();
        user.data.tokens = tokens;

        // Updates the user data
        tx.update(user).unwrap();

        // Commits so the changes to the user data are persisted
        tx.commit();
    }

    #[update]
    async fn update_user_concurrent_error(id: u32, tokens: u32) {
        // Starts a transaction
        let mut tx = db().tx();

        // Reads user data from the store
        let mut user = tx.fetch_one(&id).unwrap();
        let original_tokens = user.data.tokens;
        user.data.tokens = tokens * 10;

        // Update the user data
        tx.update(user).unwrap();

        // Calls another canister. Here the IC context is switched and the
        // state should be persisted. Nevertheless, the transaction data is
        // not persisted until commit is excuted
        {
            let canister_b_principal = CONFIG.with(|c| c.borrow().canister_b_principal);
            let _call_result: Result<(u64,), _> =
                ic_cdk::call(canister_b_principal, "get_counter", ((),)).await;            
        }

        // Here we simulate a concurrent modification of the user data
        update_user_inner(id, original_tokens);

        // We finally commit the transaction.
        // The commit will panic because another call has modified the user data concurrently.
        // Please note that the commit fails because the other call has modified the data of the very same user,
        // if another user was changed, the commit would have succeeded.
        // The state is reverted to its initial value. Even changes performed before the .await call
        // are reverted so there is no dirty state.
        tx.commit();
    }

// Enable Candid export
ic_cdk::export_candid!();

// #[cfg(test)]
// mod test {

//     use ic_exports::ic_kit::{mock_principals::alice, MockContext};

//     use super::*;

//     #[tokio::test]
//     async fn get_user_should_return_none() {
//         // Arrange
//         MockContext::new().with_id(alice()).inject();
//         let canister = CanisterA::from_principal(alice());

//         // Act
//         let result = canister_call!(canister.get_user(3), Option<Model<u32, Data>>)
//             .await
//             .unwrap();

//         // Assert
//         assert!(result.is_none())
//     }

//     #[tokio::test]
//     async fn create_user_tx_should_be_committed() {
//         // Arrange
//         MockContext::new().with_id(alice()).inject();
//         let canister = CanisterA::from_principal(alice());

//         // Act
//         let id = 111;
//         let username = "ufoscout";

//         canister_call!(canister.create_user(id, username.to_string()), ())
//             .await
//             .unwrap();
//         let result = canister_call!(canister.get_user(id), Option<Model<u32, Data>>)
//             .await
//             .unwrap();

//         // Assert
//         assert_eq!(
//             Some(Model::from((
//                 id,
//                 Data {
//                     username: username.to_string(),
//                     tokens: 0
//                 }
//             ))),
//             result
//         )
//     }

//     #[tokio::test]
//     async fn create_user_tx_should_be_rolled_back() {
//         // Arrange
//         MockContext::new().with_id(alice()).inject();
//         let canister = CanisterA::from_principal(alice());

//         // Act
//         let id = 111;
//         let username = "ufoscout";

//         let create_result =
//             canister_call!(canister.create_user_rollback(id, username.to_string()), ()).await;
//         let result = canister_call!(canister.get_user(id), Option<Model<u32, Data>>)
//             .await
//             .unwrap();

//         // Assert
//         assert!(create_result.is_ok());
//         assert!(result.is_none());
//     }

//     #[tokio::test]
//     async fn update_user_tx_should_be_committed() {
//         // Arrange
//         MockContext::new().with_id(alice()).inject();
//         let canister = CanisterA::from_principal(alice());

//         let id = 22211;
//         let username = "ufo";

//         canister_call!(canister.create_user(id, username.to_string()), ())
//             .await
//             .unwrap();

//         // Act
//         let new_tokens = 1123;
//         canister_call!(canister.update_user(id, new_tokens), ())
//             .await
//             .unwrap();
//         let result = canister_call!(canister.get_user(id), Option<Model<u32, Data>>)
//             .await
//             .unwrap();

//         // Assert
//         assert_eq!(
//             Some(Model::from((
//                 id,
//                 1,
//                 Data {
//                     username: username.to_string(),
//                     tokens: new_tokens
//                 }
//             ))),
//             result
//         )
//     }

//     #[tokio::test]
//     async fn update_user_tx_should_be_rolled_back() {
//         // Arrange
//         MockContext::new().with_id(alice()).inject();
//         let canister = CanisterA::from_principal(alice());

//         let id = 22211;
//         let username = "ufo";

//         canister_call!(canister.create_user(id, username.to_string()), ())
//             .await
//             .unwrap();

//         // Act
//         let new_tokens = 1123;

//         let update_result = std::panic::catch_unwind(|| {
//             let handle = tokio::runtime::Handle::current();
//             let _guard = handle.enter();
//             futures::executor::block_on(canister.update_user_concurrent_error(id, new_tokens))
//         });

//         let result = canister_call!(canister.get_user(id), Option<Model<u32, Data>>)
//             .await
//             .unwrap();

//         // Assert
//         assert!(update_result.is_err());
//         assert_eq!(
//             Some(Model::from((
//                 id,
//                 1,
//                 Data {
//                     username: username.to_string(),
//                     tokens: 0
//                 }
//             ))),
//             result
//         )
//     }
// }
