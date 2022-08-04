#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use rift_types::metadata::Metadata as MetadataType;
use rift_types::query::ResponseWrapper;
use rift_types::tokens::Locks;
use rift_utils::check_admin_privileges;
use url::Url;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    CollectionConfig, CollectionInfo, Config, Contracts, COLLECTION_CONFIG, COLLECTION_INFO,
    CONFIG, CONTRACTS, LOCKS, MINTED_TOKENS_PER_ADDR, MINT_MODULE_ADDR, OPERATION_LOCK, OPERATORS,
    TOKEN_IDS, TOKEN_LOCKS,
};

use cw721::{ContractInfoResponse, Cw721Execute};
use cw721_base::MintMsg;

use metadata_contract::msg::InstantiateMsg as MetadataInstantiateMsg;

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rift-token-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

const METADATA_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let locks = Locks {
        burn_lock: false,
        mint_lock: false,
        transfer_lock: false,
        send_lock: false,
    };

    let whitelist = msg
        .contracts
        .whitelist
        .and_then(|w| deps.api.addr_validate(w.as_str()).ok());
    let royalty = msg
        .contracts
        .royalty
        .and_then(|r| deps.api.addr_validate(r.as_str()).ok());
    let metadata = msg
        .contracts
        .metadata
        .and_then(|m| deps.api.addr_validate(m.as_str()).ok());
    let contracts = Contracts {
        whitelist,
        royalty,
        metadata,
    };
    CONTRACTS.save(deps.storage, &contracts)?;

    if msg.start_time.is_some() && env.block.time > msg.start_time.unwrap() {
        return Err(ContractError::InvalidStartTime {});
    };

    let collection_config = CollectionConfig {
        per_address_limit: msg.per_address_limit,
        start_time: msg.start_time,
        max_token_limit: msg.max_token_limit,
    };
    COLLECTION_CONFIG.save(deps.storage, &collection_config)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    LOCKS.save(deps.storage, &locks)?;

    TOKEN_IDS.save(deps.storage, &0)?;

    MINT_MODULE_ADDR.save(deps.storage, &info.sender)?;

    OPERATION_LOCK.save(deps.storage, &false)?;

    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    Url::parse(&msg.collection_info.image)?;

    if let Some(ref external_link) = msg.collection_info.external_link {
        Url::parse(external_link)?;
    }

    let collection_info = CollectionInfo {
        collection_type: msg.collection_info.collection_type,
        name: msg.collection_info.name.clone(),
        description: msg.collection_info.description,
        image: msg.collection_info.image,
        external_link: msg.collection_info.external_link,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    let contract_info = ContractInfoResponse {
        name: msg.collection_info.name.clone(),
        symbol: msg.token_info.symbol,
    };
    Cw721Contract::default()
        .contract_info
        .save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.token_info.minter)?;
    Cw721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.token_info.minter)
        .add_attribute("collection_name", msg.collection_info.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // LOCK MESSAGES
        ExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::UpdateTokenLock { token_id, locks } => {
            execute_update_token_locks(deps, env, info, token_id, locks)
        }
        ExecuteMsg::UpdateOperationLock { lock } => {
            execute_update_operation_lock(deps, env, info, lock)
        }
        // OPERATION MESSAGES
        ExecuteMsg::Mint { owner } => execute_mint(deps, env, info, owner),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::TransferNft {
            token_id,
            recipient,
        } => execute_transfer(deps, env, info, token_id, recipient),
        ExecuteMsg::SendNft {
            token_id,
            contract,
            msg,
        } => execute_send(deps, env, info, token_id, contract, msg),
        // CONFIG MESSAGES
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::UpdateStartTime { start_time } => {
            execute_update_start_time(deps, env, info, start_time)
        }
        ExecuteMsg::UpdateWhitelist { whitelist } => {
            execute_update_whitelist(deps, env, info, whitelist)
        }
        ExecuteMsg::UpdateRoyalty { royalty } => execute_update_royalty(deps, env, info, royalty),
        ExecuteMsg::UpdateMetadata { metadata } => {
            execute_update_metadata(deps, env, info, metadata)
        }
        // ADMIN MESSAGES
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::AdminTransferNft {
            recipient,
            token_id,
        } => execute_admin_transfer(deps, env, info, token_id, recipient),

        // CONTRACT MESSAGES
        ExecuteMsg::InitMetadataContract {
            code_id,
            metadata_type,
        } => execute_init_metadata_contract(deps, env, info, code_id, metadata_type),

        // CW721 MESSAGES
        _ => {
            let res = Cw721Contract::default().execute(deps, env, info, msg.into());
            match res {
                Ok(res) => Ok(res),
                Err(e) => Err(e.into()),
            }
        }
    }
}

pub fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    OPERATORS.save(deps.storage, &addrs)?;

    Ok(Response::new().add_attribute("action", "execute_update_operators"))
}

pub fn execute_update_locks(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    locks: Locks,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    LOCKS.save(deps.storage, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "update_locks")
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_update_token_locks(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    locks: Locks,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    TOKEN_LOCKS.save(deps.storage, &token_id, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "update_token_locks")
        .add_attribute("token_id", token_id)
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_update_operation_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    OPERATION_LOCK.save(deps.storage, &lock)?;

    Ok(Response::new()
        .add_attribute("action", "update_operation_lock")
        .add_attribute("lock", lock.to_string()))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let lock = OPERATION_LOCK.load(deps.storage)?;
    if lock {
        return Err(ContractError::MintLocked {});
    }

    let collection_config = COLLECTION_CONFIG.load(deps.storage)?;

    let locks = LOCKS.load(deps.storage)?;
    if locks.mint_lock {
        return Err(ContractError::MintLocked {});
    }

    let token_id = (TOKEN_IDS.load(deps.storage)?) + 1;

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id.to_string())?;
    if token_lock.is_some() && token_lock.unwrap().mint_lock {
        return Err(ContractError::MintLocked {});
    }

    if collection_config.max_token_limit.is_some()
        && token_id > collection_config.max_token_limit.unwrap()
    {
        return Err(ContractError::TokenLimitReached {});
    }

    let total_minted = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, &owner)?
        .unwrap_or(0);

    if collection_config.per_address_limit.is_some()
        && total_minted + 1 > collection_config.per_address_limit.unwrap()
    {
        return Err(ContractError::TokenLimitReached {});
    }

    // TODO: Check for start time here

    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: owner.clone(),
        token_uri: None,
        extension: Empty {},
    };

    MINTED_TOKENS_PER_ADDR.save(deps.storage, &owner, &(total_minted + 1))?;
    TOKEN_IDS.save(deps.storage, &token_id)?;

    let res = Cw721Contract::default().mint(deps, env, info, mint_msg);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let lock = OPERATION_LOCK.load(deps.storage)?;
    if lock {
        return Err(ContractError::BurnLocked {});
    }

    let locks = LOCKS.load(deps.storage)?;
    if locks.burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let res = Cw721Contract::default().burn(deps, env, info, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let lock = OPERATION_LOCK.load(deps.storage)?;
    if lock {
        return Err(ContractError::TransferLocked {});
    }

    let locks = LOCKS.load(deps.storage)?;
    if locks.transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let res = Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_admin_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &&env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let res = Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    contract: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let lock = OPERATION_LOCK.load(deps.storage)?;
    if lock {
        return Err(ContractError::SendLocked {});
    }

    let locks = LOCKS.load(deps.storage)?;
    if locks.send_lock {
        return Err(ContractError::SendLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().send_lock {
        return Err(ContractError::SendLocked {});
    }

    let res = Cw721Contract::default().send_nft(deps, env, info, contract, token_id, msg);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    per_address_limit: Option<u32>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let mut collection_config = COLLECTION_CONFIG.load(deps.storage)?;

    if per_address_limit.is_some() && per_address_limit.unwrap() == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    collection_config.per_address_limit = per_address_limit;
    COLLECTION_CONFIG.save(deps.storage, &collection_config)?;

    Ok(Response::new().add_attribute("action", "update_per_address_limit"))
}

fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let mut collection_config = COLLECTION_CONFIG.load(deps.storage)?;

    if collection_config.start_time.is_some()
        && env.block.time >= collection_config.start_time.unwrap()
    {
        return Err(ContractError::AlreadyStarted {});
    }

    match start_time {
        Some(time) => {
            if env.block.time > time {
                return Err(ContractError::InvalidStartTime {});
            }
            collection_config.start_time = start_time;
        }
        None => collection_config.start_time = None,
    }

    COLLECTION_CONFIG.save(deps.storage, &collection_config)?;

    Ok(Response::new().add_attribute("action", "update_start_time"))
}

fn execute_update_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelist: Option<String>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let mut contracts = CONTRACTS.load(deps.storage)?;
    contracts.whitelist = whitelist.and_then(|w| deps.api.addr_validate(w.as_str()).ok());
    CONTRACTS.save(deps.storage, &contracts)?;

    Ok(Response::new().add_attribute("action", "update_whitelist"))
}

fn execute_update_royalty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    royalty: Option<String>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let mut contracts = CONTRACTS.load(deps.storage)?;
    contracts.royalty = royalty.and_then(|r| deps.api.addr_validate(r.as_str()).ok());
    CONTRACTS.save(deps.storage, &contracts)?;

    Ok(Response::new().add_attribute("action", "update_royalty"))
}

fn execute_update_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    metadata: Option<String>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let mut contracts = CONTRACTS.load(deps.storage)?;
    contracts.metadata = metadata.and_then(|m| deps.api.addr_validate(m.as_str()).ok());
    CONTRACTS.save(deps.storage, &contracts)?;

    Ok(Response::new().add_attribute("action", "update_metadata"))
}

fn execute_init_metadata_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    metadata_type: MetadataType,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        None,
    )?;

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&MetadataInstantiateMsg {
                admin: config.admin.to_string(),
                metadata_type,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Rift Framework Metadata Contract"),
        }
        .into(),
        id: METADATA_CONTRACT_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "execute_init_metadata_contract"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Locks {} => to_binary(&query_locks(deps)?),
        QueryMsg::TokenLocks { token_id } => to_binary(&query_token_locks(deps, token_id)?),
        QueryMsg::MintedTokensPerAddress { address } => {
            to_binary(&query_minted_tokens_per_address(deps, address)?)
        }
        QueryMsg::CollectionInfo {} => to_binary(&query_collection_info(deps)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_locks(deps: Deps) -> StdResult<ResponseWrapper<Locks>> {
    let locks = LOCKS.load(deps.storage)?;
    Ok(ResponseWrapper::new("locks", locks))
}

fn query_token_locks(deps: Deps, token_id: String) -> StdResult<ResponseWrapper<Locks>> {
    let locks = TOKEN_LOCKS.load(deps.storage, &token_id)?;
    Ok(ResponseWrapper::new("locks", locks))
}

fn query_minted_tokens_per_address(deps: Deps, address: String) -> StdResult<ResponseWrapper<u32>> {
    let amount = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, &address)?
        .unwrap_or(0);
    Ok(ResponseWrapper::new("minted_tokens_per_address", amount))
}

fn query_collection_info(deps: Deps) -> StdResult<ResponseWrapper<CollectionInfo>> {
    let collection_info = COLLECTION_INFO.load(deps.storage)?;
    Ok(ResponseWrapper::new("collection_info", collection_info))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != METADATA_CONTRACT_INSTANTIATE_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let mut contracts = CONTRACTS.load(deps.storage)?;
            contracts.metadata = Some(Addr::unchecked(res.contract_address));
            CONTRACTS.save(deps.storage, &contracts)?;
            Ok(Response::default().add_attribute("action", "instantiate_metadata_reply"))
        }
        Err(_) => Err(ContractError::MetadataInstantiateError {}),
    }
}
