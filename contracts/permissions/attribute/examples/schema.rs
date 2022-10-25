use cosmwasm_schema::write_api;
use komple_attribute_permission_module::msg::{ExecuteMsg, QueryMsg};
use komple_types::shared::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
