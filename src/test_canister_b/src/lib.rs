use ic_cdk::{query, update};
use std::cell::RefCell;

thread_local! {
    static COUNTER: RefCell<u64> = RefCell::new(999_999_999);
}

/// Get the value of the counter.
#[query]
fn get_counter() -> u64 {
    COUNTER.with(|c| (*c.borrow()).clone())
}

/// Increment the value of the counter.
#[update]
fn inc() {
    COUNTER.with(|counter| *counter.borrow_mut() += 1);
}

// Enable Candid export
ic_cdk::export_candid!();
