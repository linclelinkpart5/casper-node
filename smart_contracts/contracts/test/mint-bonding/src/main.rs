#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casperlabs_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use casperlabs_types::{
    account::AccountHash, runtime_args, ApiError, ContractHash, RuntimeArgs, URef, U512,
};

const ARG_AMOUNT: &str = "amount";
const ARG_PURSE: &str = "purse";
const ARG_ENTRY_POINT: &str = "entry_point";
const ARG_BOND: &str = "bond";
const ARG_UNBOND: &str = "unbond";
const ARG_RELEASE: &str = "release";
const ARG_ACCOUNT_HASH: &str = "account_hash";
const TEST_BOND_FROM_MAIN_PURSE: &str = "bond-from-main-purse";
const TEST_SEED_NEW_ACCOUNT: &str = "seed_new_account";
const METHOD_RELEASE_FOUNDER_STAKE: &str = "release_founder_stake";

#[repr(u16)]
enum Error {
    UnableToSeedAccount,
    UnknownCommand,
}

#[no_mangle]
pub extern "C" fn call() {
    let command: String = runtime::get_named_arg(ARG_ENTRY_POINT);

    match command.as_str() {
        ARG_BOND => bond(),
        ARG_UNBOND => unbond(),
        ARG_RELEASE => release(),
        TEST_BOND_FROM_MAIN_PURSE => bond_from_main_purse(),
        TEST_SEED_NEW_ACCOUNT => seed_new_account(),
        _ => runtime::revert(ApiError::User(Error::UnknownCommand as u16)),
    }
}

fn bond() {
    let mint_contract_hash = system::get_mint();
    // Creates new purse with desired amount based on main purse and sends funds
    let amount = runtime::get_named_arg(ARG_AMOUNT);
    let bonding_purse = system::create_purse();

    system::transfer_from_purse_to_purse(account::get_main_purse(), bonding_purse, amount)
        .unwrap_or_revert();

    bonding(mint_contract_hash, amount, bonding_purse);
}

fn bond_from_main_purse() {
    let mint_contract_hash = system::get_mint();
    let amount = runtime::get_named_arg(ARG_AMOUNT);
    bonding(mint_contract_hash, amount, account::get_main_purse());
}

fn bonding(mint: ContractHash, bond_amount: U512, bonding_purse: URef) {
    let args = runtime_args! {
        ARG_AMOUNT => bond_amount,
        ARG_PURSE => bonding_purse,
    };

    let (_purse, _quantity): (URef, U512) = runtime::call_contract(mint, ARG_BOND, args);
}

fn unbond() {
    let mint_contract_hash = system::get_mint();
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    unbonding(mint_contract_hash, amount);
}

fn unbonding(mint: ContractHash, unbond_amount: U512) -> (URef, U512) {
    let args = runtime_args! {
        ARG_AMOUNT => unbond_amount,
    };
    runtime::call_contract(mint, ARG_UNBOND, args)
}

fn release() {
    let account_hash = runtime::get_named_arg("validator_account_hash");
    release_founder_stake(account_hash);
}

fn release_founder_stake(account_hash: AccountHash) {
    let args = runtime_args! {
        "validator_account_hash" => account_hash,
    };
    let _result: bool =
        runtime::call_contract(system::get_mint(), METHOD_RELEASE_FOUNDER_STAKE, args);
}

fn seed_new_account() {
    let source = account::get_main_purse();
    let target: AccountHash = runtime::get_named_arg(ARG_ACCOUNT_HASH);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    system::transfer_from_purse_to_account(source, target, amount)
        .unwrap_or_revert_with(ApiError::User(Error::UnableToSeedAccount as u16));
}
