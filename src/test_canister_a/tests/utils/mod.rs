use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;

use candid::{CandidType, Encode, Principal};
use pocket_ic::nonblocking::PocketIc;
use test_canister_a::InitArgs;
use ic_mple_pocket_ic::{*};

pub fn alice() -> Principal {
    Principal::from_text("sgymv-uiaaa-aaaaa-aaaia-cai").unwrap()
}

pub struct PocketIcTestContext {
    pub client: PocketIc,
    pub canister_a_principal: Principal,
    // pub canister_a_args: InitArgs,
    // pub canister_b_principal: Principal,
}

impl PocketIcTestContext {

    pub async fn get_counter(&self, sender: Principal) -> u64 {
        let args = &();
        query_call(&self.client, self.canister_a_principal, sender, "get_counter", args).await
    }

    pub async fn get_counter_from_another_canister(&self, sender: Principal) -> u64 {
        let args = &();
        update_call(
            &self.client,
            self.canister_a_principal,
            sender,
            "get_counter_from_another_canister",
            args,
        ).await
    }

    pub async fn new() -> Self {
        let client = get_pocket_ic_client().build_async().await;
        let canister_b_principal = deploy_canister(&client, get_canister_b_bytecode(), &()).await;
        let canister_a_args = InitArgs {
            canister_b_principal,
        };
        let canister_a_principal =
            deploy_canister(&client, get_canister_a_bytecode(), &canister_a_args).await;
    
        PocketIcTestContext {
            client: client,
            canister_a_principal,
            // canister_a_args,
            // canister_b_principal,
        }
    }
}


async fn deploy_canister<T: CandidType>(client: &PocketIc, bytecode: Vec<u8>, args: &T) -> Principal {
    let args = Encode!(args).expect("failed to encode item to candid");
    let canister = client.create_canister().await;
    client.add_cycles(canister, 10_u128.pow(12)).await;
    client.install_canister(canister, bytecode, args, None).await;
    canister
}


fn get_canister_a_bytecode() -> Vec<u8> {
    static CANISTER_BYTECODE: OnceLock<Vec<u8>> = OnceLock::new();
    CANISTER_BYTECODE
        .get_or_init(|| load_wasm_bytes("../target/wasm32-unknown-unknown/release/test_canister_a.wasm"))
        .to_owned()
}

fn get_canister_b_bytecode() -> Vec<u8> {
    static CANISTER_BYTECODE: OnceLock<Vec<u8>> = OnceLock::new();
    CANISTER_BYTECODE
        .get_or_init(|| load_wasm_bytes("../target/wasm32-unknown-unknown/release/test_canister_b.wasm"))
        .to_owned()
}
