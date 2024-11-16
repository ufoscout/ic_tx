use utils::PocketIcTestContext;

mod utils;

#[tokio::test]
async fn get_user_should_return_none() {
    // Arrange
    let ctx = PocketIcTestContext::new().await;

    // Act
    ctx.b_get_counter().await;
}

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
