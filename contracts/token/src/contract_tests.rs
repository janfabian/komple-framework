#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate},
        msg::ExecuteMsg,
        state::Locks,
        ContractError,
    };
    use cosmwasm_std::{
        attr,
        testing::{mock_dependencies, mock_env, mock_info},
        Empty,
    };
    use cw721_base::{msg::InstantiateMsg, MintMsg};

    #[test]
    fn update_locks() {
        let admin_info = mock_info("admin", &vec![]);
        let random_info = mock_info("random", &vec![]);
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            name: "Test Project".to_string(),
            symbol: "TP".to_string(),
            minter: "admin".to_string(),
        };

        let res = instantiate(deps.as_mut(), mock_env(), admin_info.clone(), msg);
        assert!(res.is_ok());

        let locks = Locks {
            burn_lock: false,
            mint_lock: true,
            transfer_lock: true,
            send_lock: false,
        };
        let msg = ExecuteMsg::UpdateLocks { locks };

        let res = execute(deps.as_mut(), mock_env(), random_info.clone(), msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        let res = execute(deps.as_mut(), mock_env(), admin_info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "update_locks"),
                attr("mint_lock", "true"),
                attr("burn_lock", "false"),
                attr("transfer_lock", "true"),
                attr("send_lock", "false"),
            ]
        );

        let msg: ExecuteMsg<Empty> = ExecuteMsg::Mint(MintMsg {
            token_id: "1".to_string(),
            owner: "random".to_string(),
            token_uri: None,
            extension: Empty {},
        });
        let res = execute(deps.as_mut(), mock_env(), random_info.clone(), msg).unwrap_err();
        assert_eq!(res, ContractError::MintLocked {});

        let msg: ExecuteMsg<Empty> = ExecuteMsg::TransferNft {
            recipient: "admin".to_string(),
            token_id: "1".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), random_info, msg).unwrap_err();
        assert_eq!(res, ContractError::TransferLocked {});
    }
}
