pub mod argument_parser;
pub mod component;
pub mod fee_claim;
pub mod hashing;
pub mod serde_with;
pub mod shard_id;
pub mod substate;
pub mod template;
pub mod transaction;
pub mod types;

use std::str::FromStr;

use component::get_account_address_from_public_key;
use shard_id::ShardId;
use substate::SubstateAddress;
use tari_crypto::keys::PublicKey;
use tari_crypto::ristretto::{RistrettoPublicKey, RistrettoSecretKey};
use tari_crypto::tari_utilities::hex::Hex;
use tari_crypto::tari_utilities::ByteArray;
use tari_template_lib::args;
use tari_template_lib::prelude::{
    Amount, NonFungibleAddress, ResourceAddress, RistrettoPublicKeyBytes, TemplateAddress,
};
use transaction::instruction::Instruction;
use transaction::transaction::Transaction;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn build_ristretto_private_key(ecdsa_private_key: &str) -> Result<String, JsError> {
    let private_key = RistrettoSecretKey::from_hex(ecdsa_private_key)?;
    Ok(private_key.to_hex())
}

#[wasm_bindgen]
pub fn build_ristretto_public_key(ecdsa_private_key: &str) -> Result<String, JsError> {
    let private_key = RistrettoSecretKey::from_hex(ecdsa_private_key)?;
    let public_key = RistrettoPublicKey::from_secret_key(&private_key);
    Ok(public_key.to_hex())
}

#[wasm_bindgen]
pub fn get_account_component_address(public_key: &str) -> Result<String, JsError> {
    let account_address = get_account_address_from_public_key(&public_key)?;
    Ok(account_address.to_string())
}

#[wasm_bindgen]
pub fn create_transaction(
    account_private_key_hex: &str,
    instructions_js: JsValue,
    input_refs_js: JsValue,
) -> Result<JsValue, JsError> {
    let account_private_key = RistrettoSecretKey::from_hex(account_private_key_hex)?;
    let instructions: Vec<Instruction> = serde_wasm_bindgen::from_value(instructions_js)?;
    let input_refs: Vec<ShardId> = serde_wasm_bindgen::from_value(input_refs_js)?;

    let transaction = Transaction::builder()
        .with_fee_instructions(instructions.to_vec())
        .with_input_refs(input_refs.to_vec())
        .sign(&account_private_key)
        .build();

    Ok(serde_wasm_bindgen::to_value(&transaction)?)
}

#[wasm_bindgen]
pub fn create_transfer_transaction(
    source_private_key: &str,
    destination_public_key_hex: &str,
    create_destination_account: bool,
    resource_address: &str,
    amount: i64,
    fee: i64,
) -> Result<JsValue, JsError> {
    let source_private_key = RistrettoSecretKey::from_hex(source_private_key)?;
    let source_public_key = RistrettoPublicKey::from_secret_key(&source_private_key);
    let source_account_address = get_account_address_from_public_key(&source_public_key.to_hex())?;

    let destination_account_address = get_account_address_from_public_key(destination_public_key_hex)?;
    let destination_public_key = RistrettoPublicKey::from_hex(destination_public_key_hex)?;

    let mut instructions = vec![
        Instruction::CallMethod {
            component_address: source_account_address,
            method: "withdraw".to_string(),
            args: args![
                ResourceAddress::from_str(resource_address)?,
                Amount::new(amount)
            ],
        },
        Instruction::PutLastInstructionOutputOnWorkspace {
            key: b"bucket".to_vec(),
        },
    ];

    if create_destination_account {
        let owner_token = NonFungibleAddress::from_public_key(
            RistrettoPublicKeyBytes::from_bytes(destination_public_key.as_bytes()).unwrap(),
        );
        let template_address: TemplateAddress = TemplateAddress::from_array([0; 32]);
        instructions.push(Instruction::CallFunction {
            template_address,
            function: "create".to_string(),
            args: args![owner_token],
        });
    }

    instructions.push(Instruction::CallMethod {
        component_address: destination_account_address,
        method: "deposit".to_string(),
        args: args![Workspace("bucket")],
    });
    instructions.push(Instruction::CallMethod {
        component_address: source_account_address,
        method: "pay_fee".to_string(),
        args: args![Amount::new(fee)],
    });

    let resource_address_obj = ResourceAddress::from_str(resource_address)?;
    let resource_substate = SubstateAddress::Resource(resource_address_obj);
    let resource_shard_id = ShardId::from_address(&resource_substate, 0);
    let input_refs = vec![resource_shard_id];

    let transaction = Transaction::builder()
        .with_fee_instructions(instructions.to_vec())
        .with_input_refs(input_refs)
        .sign(&source_private_key)
        .build();

    Ok(serde_wasm_bindgen::to_value(&transaction)?)
}

#[wasm_bindgen]
pub fn create_free_test_coins_transaction(
    is_new_account: bool,
    account_private_key: &str,
    amount: i64,
    fee: i64,
) -> Result<JsValue, JsError> {
    let account_private_key = RistrettoSecretKey::from_hex(account_private_key)?;
    let account_public_key = RistrettoPublicKey::from_secret_key(&account_private_key);
    let account_component_address =
        get_account_address_from_public_key(&account_public_key.to_hex())?;

    let mut instructions = vec![
        Instruction::CreateFreeTestCoins {
            revealed_amount: Amount::new(amount),
            output: None,
        },
        Instruction::PutLastInstructionOutputOnWorkspace {
            key: b"free_test_coins".to_vec(),
        },
    ];

    if is_new_account {
        let owner_token = NonFungibleAddress::from_public_key(
            RistrettoPublicKeyBytes::from_bytes(account_public_key.as_bytes()).unwrap(),
        );
        let template_address: TemplateAddress = TemplateAddress::from_array([0; 32]);
        instructions.push(Instruction::CallFunction {
            template_address,
            function: "create_with_bucket".to_string(),
            args: args![owner_token, Workspace("free_test_coins")],
        });
    } else {
        instructions.push(Instruction::CallMethod {
            component_address: account_component_address,
            method: "deposit".to_string(),
            args: args![Workspace("free_test_coins")],
        });
    }

    // Pay fees from the account
    instructions.push(Instruction::CallMethod {
        component_address: account_component_address,
        method: "pay_fee".to_string(),
        args: args![Amount::new(fee)],
    });

    let transaction = Transaction::builder()
        .with_fee_instructions(instructions.to_vec())
        .sign(&account_private_key)
        .build();

    Ok(serde_wasm_bindgen::to_value(&transaction)?)
}
