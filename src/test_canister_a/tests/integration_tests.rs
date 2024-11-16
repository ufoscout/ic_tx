use ic_tx::model::Model;
use test_canister_a::Data;
use utils::PocketIcTestContext;

mod utils;

    #[tokio::test]
    async fn create_user_tx_should_be_committed() {
        // Arrange
        let ctx = PocketIcTestContext::new().await;
        let id = 111;
        let username = "ufoscout";
        
        // Act
        ctx.create_user(id, username.to_string()).await;
        let result = ctx.get_user(id).await;

        // Assert
        assert_eq!(
            Some(Model::from((
                id,
                Data {
                    username: username.to_string(),
                    tokens: 0
                }
            ))),
            result
        )
    }

    #[tokio::test]
    async fn create_user_tx_should_be_rolled_back() {
        // Arrange
        let ctx = PocketIcTestContext::new().await;

        let id = 111;
        let username = "ufoscout";
        
        // Act
        ctx.create_user_rollback(id, username.to_string()).await;
        let result = ctx.get_user(id).await;

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn update_user_tx_should_be_committed() {
        // Arrange
        let ctx = PocketIcTestContext::new().await;

        let id = 22211;
        let username = "ufo";

        ctx.create_user(id, username.to_string()).await;
        let new_tokens = 1123;

        // Act
        ctx.update_user(id, new_tokens).await;
        let result = ctx.get_user(id).await;

        // Assert
        assert_eq!(
            Some(Model::from((
                id,
                1,
                Data {
                    username: username.to_string(),
                    tokens: new_tokens
                }
            ))),
            result
        )
    }

    #[tokio::test]
    async fn update_user_tx_should_be_rolled_back() {
        // Arrange
        let ctx = PocketIcTestContext::new().await;

        let id = 22211;
        let username = "ufo";

        ctx.create_user(id, username.to_string()).await;

        // Act
        let new_tokens = 1123;
        let update_result = ctx.update_user_concurrent_error(id, new_tokens).await;

        let result = ctx.get_user(id).await;

        // Assert
        assert!(update_result.is_err());
        assert_eq!(
            Some(Model::from((
                id,
                0,
                Data {
                    username: username.to_string(),
                    tokens: 0
                }
            ))),
            result
        )
    }
