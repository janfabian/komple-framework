use cosmwasm_std::{Addr, Coin, Decimal, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_contract::msg::InstantiateMsg as FeeContractInstantiateMsg;
use komple_hub_module::msg::{ExecuteMsg, InstantiateMsg};
use komple_token_module::{
    msg::{
        ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg, TokenInfo,
    },
    state::{BundleInfo, Contracts},
};
use komple_types::{
    bundle::Bundles, metadata::Metadata as MetadataType, module::Modules, permission::Permissions,
    query::ResponseWrapper,
};
use komple_utils::{query_bundle_address, query_module_address};
use marketplace_module::msg::ExecuteMsg as MarketplaceExecuteMsg;
use metadata_contract::msg::ExecuteMsg as MetadataExecuteMsg;
use metadata_contract::state::{MetaInfo, Trait};
use mint_module::msg::ExecuteMsg as MintExecuteMsg;
use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const RANDOM_2: &str = "juno..random2";
pub const NATIVE_DENOM: &str = "denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_hub_module::contract::execute,
        komple_hub_module::contract::instantiate,
        komple_hub_module::contract::query,
    )
    .with_reply(komple_hub_module::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mint_module::contract::execute,
        mint_module::contract::instantiate,
        mint_module::contract::query,
    )
    .with_reply(mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        permission_module::contract::execute,
        permission_module::contract::instantiate,
        permission_module::contract::query,
    );
    Box::new(contract)
}

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_token_module::contract::execute,
        komple_token_module::contract::instantiate,
        komple_token_module::contract::query,
    )
    .with_reply(komple_token_module::contract::reply);
    Box::new(contract)
}

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        merge_module::contract::execute,
        merge_module::contract::instantiate,
        merge_module::contract::query,
    );
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        marketplace_module::contract::execute,
        marketplace_module::contract::instantiate,
        marketplace_module::contract::query,
    );
    Box::new(contract)
}

pub fn metadata_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        metadata_contract::contract::execute,
        metadata_contract::contract::instantiate,
        metadata_contract::contract::query,
    );
    Box::new(contract)
}

pub fn fee_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_contract::contract::execute,
        komple_fee_contract::contract::instantiate,
        komple_fee_contract::contract::query,
    );
    Box::new(contract)
}

pub fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM_2),
                vec![Coin {
                    denom: TEST_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

pub fn proper_instantiate(app: &mut App) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let msg = InstantiateMsg {
        name: "Test Hub".to_string(),
        description: "Test Hub".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let hub_module_addr = app
        .instantiate_contract(hub_code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap();

    hub_module_addr
}

pub fn setup_mint_module(app: &mut App, hub_addr: Addr) {
    let mint_module_code_id = app.store_code(mint_module());

    let msg = ExecuteMsg::InitMintModule {
        code_id: mint_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_merge_module(app: &mut App, hub_addr: Addr) {
    let merge_module_code_id = app.store_code(merge_module());

    let msg = ExecuteMsg::InitMergeModule {
        code_id: merge_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_permission_module(app: &mut App, hub_addr: Addr) {
    let permission_module_code_id = app.store_code(permission_module());

    let msg = ExecuteMsg::InitPermissionModule {
        code_id: permission_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_marketplace_module(app: &mut App, hub_addr: Addr) {
    let marketplace_module_code_id = app.store_code(marketplace_module());

    let msg = ExecuteMsg::InitMarketplaceModule {
        code_id: marketplace_module_code_id,
        native_denom: NATIVE_DENOM.to_string(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_fee_contract(app: &mut App) -> Addr {
    let fee_code_id = app.store_code(fee_contract());

    let msg = FeeContractInstantiateMsg {
        komple_address: ADMIN.to_string(),
        payment_address: "juno..community".to_string(),
    };
    let fee_contract_addr = app
        .instantiate_contract(
            fee_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &vec![],
            "test",
            None,
        )
        .unwrap();

    fee_contract_addr
}

pub fn setup_all_modules(app: &mut App, hub_addr: Addr) {
    setup_mint_module(app, hub_addr.clone());
    setup_merge_module(app, hub_addr.clone());
    setup_permission_module(app, hub_addr.clone());
    setup_marketplace_module(app, hub_addr.clone());
}

pub fn create_bundle(
    app: &mut App,
    mint_module_addr: Addr,
    token_module_code_id: u64,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    bundle_type: Bundles,
    linked_bundles: Option<Vec<u32>>,
    unit_price: Option<Uint128>,
    max_token_limit: Option<u32>,
    royalty_share: Option<Decimal>,
) {
    let bundle_info = BundleInfo {
        bundle_type,
        name: "Test Bundle".to_string(),
        description: "Test Bundle".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let msg = MintExecuteMsg::CreateBundle {
        code_id: token_module_code_id,
        token_instantiate_msg: TokenInstantiateMsg {
            admin: ADMIN.to_string(),
            bundle_info,
            token_info,
            per_address_limit,
            start_time,
            unit_price,
            native_denom: NATIVE_DENOM.to_string(),
            max_token_limit,
            royalty_share,
        },
        linked_bundles,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, bundle_id: u32, sender: &str) {
    let msg = MintExecuteMsg::Mint {
        bundle_id,
        metadata_id: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_mint_module_operators(app: &mut App, mint_module_addr: Addr, addrs: Vec<String>) {
    let msg = MintExecuteMsg::UpdateOperators { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_token_module_operators(app: &mut App, token_module_addr: Addr, addrs: Vec<String>) {
    let msg = TokenExecuteMsg::UpdateOperators { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_module_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg = TokenExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn add_permission_for_module(
    app: &mut App,
    permission_module_addr: Addr,
    module: Modules,
    permissions: Vec<Permissions>,
) {
    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module,
        permissions,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr,
            &msg,
            &vec![],
        )
        .unwrap();
}

pub fn link_bundles(
    app: &mut App,
    mint_module_addr: Addr,
    bundle_id: u32,
    linked_bundles: Vec<u32>,
) {
    let msg = MintExecuteMsg::UpdateLinkedBundles {
        bundle_id,
        linked_bundles,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn get_modules_addresses(app: &mut App, hub_addr: &Addr) -> (Addr, Addr, Addr, Addr) {
    let mint_module_addr: Addr;
    let merge_module_addr: Addr;
    let permission_module_addr: Addr;
    let marketplace_module_addr: Addr;

    let res = query_module_address(&app.wrap(), hub_addr, Modules::Mint);
    mint_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), hub_addr, Modules::Merge);
    merge_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), hub_addr, Modules::Permission);
    permission_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), hub_addr, Modules::Marketplace);
    marketplace_module_addr = res.unwrap();

    // println!("");
    // println!("mint_module_addr: {}", mint_module_addr);
    // println!("merge_module_addr: {}", merge_module_addr);
    // println!("permission_module_addr: {}", permission_module_addr);
    // println!("");

    (
        mint_module_addr,
        merge_module_addr,
        permission_module_addr,
        marketplace_module_addr,
    )
}

pub fn setup_marketplace_listing(
    app: &mut App,
    hub_addr: &Addr,
    bundle_id: u32,
    token_id: u32,
    price: Uint128,
) {
    let (mint_module_addr, _, _, marketplace_module_addr) = get_modules_addresses(app, &hub_addr);

    let bundle_addr = query_bundle_address(&app.wrap(), &mint_module_addr, &bundle_id).unwrap();

    setup_token_module_operators(
        app,
        bundle_addr.clone(),
        vec![marketplace_module_addr.to_string()],
    );

    let msg = MarketplaceExecuteMsg::ListFixedToken {
        bundle_id,
        token_id,
        price,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(USER),
            marketplace_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
}

pub fn setup_metadata_contract(
    app: &mut App,
    token_module_addr: Addr,
    metadata_type: MetadataType,
) -> Addr {
    let metadata_code_id = app.store_code(metadata_contract());

    let msg = TokenExecuteMsg::InitMetadataContract {
        code_id: metadata_code_id,
        metadata_type,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
        .unwrap();

    let res: ResponseWrapper<Contracts> = app
        .wrap()
        .query_wasm_smart(token_module_addr.clone(), &TokenQueryMsg::Contracts {})
        .unwrap();
    res.data.metadata.unwrap()
}

pub fn setup_metadata(app: &mut App, metadata_contract_addr: Addr) {
    let meta_info = MetaInfo {
        image: Some("https://some-image.com".to_string()),
        external_url: None,
        description: Some("Some description".to_string()),
        youtube_url: None,
        animation_url: None,
    };
    let attributes = vec![
        Trait {
            trait_type: "trait_1".to_string(),
            value: "value_1".to_string(),
        },
        Trait {
            trait_type: "trait_2".to_string(),
            value: "value_2".to_string(),
        },
    ];
    let msg = MetadataExecuteMsg::AddMetadata {
        meta_info,
        attributes,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            metadata_contract_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
}
