use cosmwasm_schema::cw_serde;

use cw721_base::Extension;

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub cw721_address: Option<Addr>,
    pub max_tokens: u32,
    pub unit_price: Coin,
    pub name: String,
    pub symbol: String,
    pub token_uri: String,
    pub extension: Extension,
    pub unused_token_id: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");
