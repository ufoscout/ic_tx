use std::sync::OnceLock;

use candid::{CandidType, Encode, Principal};
use ic_mple_client::*;
use ic_mple_pocket_ic::{pocket_ic::nonblocking::PocketIc, *};
use ic_tx::model::Model;
use test_canister_a::{Data, InitArgs};

pub fn alice() -> Principal {
    Principal::from_text("sgymv-uiaaa-aaaaa-aaaia-cai").unwrap()
}

#[derive(Clone)]
pub struct PocketIcTestContext {
    pub client: PocketIcClient,
    // canister_a_principal: Principal,
    // pub canister_a_args: InitArgs,
    // canister_b_principal: Principal,
}

impl PocketIcTestContext {

    pub async fn create_user(&self, id: u32, username: String) {
        self.client.update(
            "create_user",
            (id, username)
        )
        .await.unwrap()
    }

    pub async fn create_user_rollback(&self, id: u32, username: String) {
        self.client.update(
            "create_user_rollback",
            (id, username)
        )
        .await.unwrap()
    }

    pub async fn update_user(&self, id: u32, tokens: u32) {
        self.client.update(
            "update_user",
            (id, tokens)
        ).await.unwrap()
    }

    pub async fn update_user_concurrent_error(&self, id: u32, tokens: u32) -> CanisterClientResult<()> {
        self.client.update(
            "update_user_concurrent_error",
            (id, tokens)
        ).await
    }

    pub async fn get_user(&self, id: u32) -> Option<Model<u32, Data>> {
        self.client.query(
            "get_user",
            (id, ),
        )
        .await.unwrap()
    }

    pub async fn new() -> Self {
        let client = get_pocket_ic_client()
            .with_nns_subnet()
            .with_ii_subnet()
            .with_application_subnet()
            .build_async()
            .await;

        let canister_b_principal = deploy_canister(&client, get_canister_b_bytecode(), &()).await;
        let canister_a_args = InitArgs {
            canister_b_principal,
        };
        let canister_a_principal =
            deploy_canister(&client, get_canister_a_bytecode(), &canister_a_args).await;

        

        PocketIcTestContext {
            client: PocketIcClient::from_client(client, canister_a_principal, alice()),
            // canister_a_principal,
            // canister_a_args,
            // canister_b_principal,
        }
    }
}

async fn deploy_canister<T: CandidType>(
    client: &PocketIc,
    bytecode: Vec<u8>,
    args: &T,
) -> Principal {
    let args = Encode!(args).expect("failed to encode item to candid");
    let canister = client.create_canister().await;
    client.add_cycles(canister, 10_u128.pow(12)).await;
    client
        .install_canister(canister, bytecode, args, None)
        .await;
    canister
}

fn get_canister_a_bytecode() -> Vec<u8> {
    static CANISTER_BYTECODE: OnceLock<Vec<u8>> = OnceLock::new();
    CANISTER_BYTECODE
        .get_or_init(|| {
            load_wasm_bytes("../../target/wasm32-unknown-unknown/release/test_canister_a.wasm")
        })
        .to_owned()
}

fn get_canister_b_bytecode() -> Vec<u8> {
    static CANISTER_BYTECODE: OnceLock<Vec<u8>> = OnceLock::new();
    CANISTER_BYTECODE
        .get_or_init(|| {
            load_wasm_bytes("../../target/wasm32-unknown-unknown/release/test_canister_b.wasm")
        })
        .to_owned()
}
