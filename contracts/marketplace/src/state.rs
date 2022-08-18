use komple_types::{
    marketplace::FIXED_LISTING_NAMESPACE,
    shared::{CONFIG_NAMESPACE, CONTROLLER_ADDR_NAMESPACE},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    // General marketplace fee from every buy/sell
    pub fee_percentage: Decimal,
    pub native_denom: String,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const CONTROLLER_ADDR: Item<Addr> = Item::new(CONTROLLER_ADDR_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FixedListing {
    pub bundle_id: u32,
    pub token_id: u32,
    pub price: Uint128,
    pub owner: Addr,
}
pub const FIXED_LISTING: Map<(u32, u32), FixedListing> = Map::new(FIXED_LISTING_NAMESPACE);
