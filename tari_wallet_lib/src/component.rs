use tari_template_lib::{prelude::ComponentAddress, Hash};
use wasm_bindgen::JsError;

use crate::{template::TemplateAddress, hashing::{EngineHashDomainLabel, hasher}};


pub fn get_account_address_from_public_key(public_key: &str) -> Result<ComponentAddress, JsError> {
    let destination_component_id = Hash::from_hex(public_key)?;
    const ACCOUNT_TEMPLATE_ADDRESS: TemplateAddress = TemplateAddress::from_array([0; 32]);
    let account_address = new_component_address_from_parts(&ACCOUNT_TEMPLATE_ADDRESS, &destination_component_id);

    Ok(account_address)
}

pub fn get_account_nft_address_from_public_key(public_key: &str) -> Result<ComponentAddress, JsError> {
    let destination_component_id = Hash::from_hex(public_key)?;
    const ACCOUNT_NFT_TEMPLATE_ADDRESS: TemplateAddress = TemplateAddress::from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
    ]);
    let account_nft_address = new_component_address_from_parts(&ACCOUNT_NFT_TEMPLATE_ADDRESS, &destination_component_id);

    Ok(account_nft_address)
}

pub fn new_component_address_from_parts(template_address: &TemplateAddress, component_id: &Hash) -> ComponentAddress {
    let address = hasher(EngineHashDomainLabel::ComponentAddress)
        .chain(template_address)
        .chain(component_id)
        .result();
    ComponentAddress::new(address)
}