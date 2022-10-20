#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, from_binary};
use cw2::set_contract_version;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::HUB_ADDR_NAMESPACE;
use komple_utils::event::EventHelper;
use komple_utils::storage::StorageHelper;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, LinkMsg};
use crate::state::{Config, CONFIG, PERMISSION_MODULE_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:link-permission";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config {
        admin
    };
    CONFIG.save(deps.storage, &config)?;

    PERMISSION_MODULE_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_link_permission_module")
            .add_attribute("action", "instantiate")
            .add_attribute("admin", config.admin)
            .add_attribute("permission_module_addr", info.sender)
            .get(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Check { data } => execute_check(deps, env, info, data),
    }
}

pub fn execute_check(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    data: Binary,
) -> Result<Response, ContractError> {
    let permission_addr = PERMISSION_MODULE_ADDR.load(deps.storage)?;
    let hub_addr =
        StorageHelper::query_storage::<Addr>(&deps.querier, &permission_addr, HUB_ADDR_NAMESPACE)?;
    let mint_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr.unwrap(), Modules::Mint)?;

    let msgs: Vec<LinkMsg> = from_binary(&data)?;

    for msg in msgs {
        if msg.collection_ids.is_empty() {
            return Err(ContractError::EmptyCollections {});
        };

        // Get the linked collections
        let linked_collections = StorageHelper::query_linked_collections(&deps.querier, &mint_module_addr, msg.collection_id)?;
        for collection_id in linked_collections {
            if !msg.collection_ids.contains(&collection_id) {
                return Err(ContractError::LinkedCollectionNotFound {  });
            }
        }
    }

    Ok(Response::new().add_event(
        EventHelper::new("komple_attribute_permission_module")
            .add_attribute("action", "check")
            .get(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper {
        query: "config".to_string(),
        data: config,
    })
}
