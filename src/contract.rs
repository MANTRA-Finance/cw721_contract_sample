use std::marker::PhantomData;

use cosmwasm_std::{ensure, entry_point, to_json_binary};
use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw721_base::helpers::Cw721Contract;
use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, Extension, InstantiateMsg as Cw721InstantiateMsg};
use cw_utils::{must_pay, parse_reply_instantiate_data};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aura-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ensure!(
        msg.unit_price.amount > Uint128::zero(),
        ContractError::InvalidUnitPrice {}
    );
    ensure!(msg.max_tokens > 0, ContractError::InvalidMaxTokens {});

    let config = Config {
        cw721_address: None,
        unit_price: msg.unit_price,
        max_tokens: msg.max_tokens,
        owner: info.sender,
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        token_uri: msg.token_uri.clone(),
        extension: msg.extension.clone(),
        unused_token_id: 0,
    };

    CONFIG.save(deps.storage, &config)?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_json_binary(&Cw721InstantiateMsg {
                name: msg.name.clone(),
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            })?,
            funds: vec![],
            admin: None,
            label: String::from("Instantiate fixed price NFT contract"),
        }
        .into(),
        id: INSTANTIATE_TOKEN_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    ensure!(
        config.cw721_address == None,
        ContractError::Cw721AlreadyLinked {}
    );
    ensure!(
        msg.id == INSTANTIATE_TOKEN_REPLY_ID,
        ContractError::InvalidTokenReplyId {}
    );

    let reply = parse_reply_instantiate_data(msg).unwrap();
    config.cw721_address = Addr::unchecked(reply.contract_address).into();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        //No migrate implemented
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint => execute_mint(deps, info),
    }
}

pub fn execute_mint(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.cw721_address == None {
        return Err(ContractError::Uninitialized {});
    }

    if config.unused_token_id >= config.max_tokens {
        return Err(ContractError::SoldOut {});
    }

    let amount = must_pay(&info, &config.unit_price.denom)?;

    if amount != config.unit_price.amount {
        return Err(ContractError::WrongPaymentAmount {});
    }

    let mint_msg = Cw721ExecuteMsg::<Extension, Empty>::Mint {
        token_id: config.unused_token_id.to_string(),
        owner: info.sender.into_string(),
        token_uri: config.token_uri.clone().into(),
        extension: config.extension.clone(),
    };

    match config.cw721_address.clone() {
        Some(cw721) => {
            let callback =
                Cw721Contract::<Empty, Empty>(cw721, PhantomData, PhantomData).call(mint_msg)?;
            config.unused_token_id += 1;
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new().add_message(callback))
        }
        None => Err(ContractError::Cw721NotLinked {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        cw721_address: config.cw721_address,
        max_tokens: config.max_tokens,
        unit_price: config.unit_price,
        name: config.name,
        symbol: config.symbol,
        token_uri: config.token_uri,
        extension: config.extension,
        unused_token_id: config.unused_token_id,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_json, to_json_binary, Coin, SubMsgResponse, SubMsgResult};
    use prost::Message;

    use super::*;

    const NFT_CONTRACT_ADDR: &str = "nftcontract";

    #[derive(Clone, PartialEq, Message)]
    struct MsgInstantiateContractResponse {
        #[prost(string, tag = "1")]
        pub contract_address: ::prost::alloc::string::String,
        #[prost(bytes, tag = "2")]
        pub data: ::prost::alloc::vec::Vec<u8>,
    }

    #[test]
    fn initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: "owner".to_string(),
            max_tokens: 1,
            unit_price: Coin {
                denom: "uom".to_string(),
                amount: Uint128::one(),
            },
            name: String::from("FirstFT"),
            symbol: String::from("FFT"),
            token_code_id: 10u64,
            token_uri: String::from("https://ipfs.io/ipfs/Q"),
            extension: None,
        };

        let info = mock_info("owner", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg {
                msg: WasmMsg::Instantiate {
                    code_id: msg.token_code_id,
                    msg: to_json_binary(&Cw721InstantiateMsg {
                        name: msg.name.clone(),
                        symbol: msg.symbol.clone(),
                        minter: MOCK_CONTRACT_ADDR.to_string(),
                    })
                    .unwrap(),
                    funds: vec![],
                    admin: None,
                    label: String::from("Instantiate fixed price NFT contract"),
                }
                .into(),
                id: INSTANTIATE_TOKEN_REPLY_ID,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }]
        );

        let instantiate_reply = MsgInstantiateContractResponse {
            contract_address: "nftcontract".to_string(),
            data: vec![2u8; 32769],
        };
        let mut encoded_instantiate_reply =
            Vec::<u8>::with_capacity(instantiate_reply.encoded_len());
        instantiate_reply
            .encode(&mut encoded_instantiate_reply)
            .unwrap();

        let reply_msg = Reply {
            id: INSTANTIATE_TOKEN_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(encoded_instantiate_reply.into()),
            }),
        };
        reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

        let query_msg = QueryMsg::GetConfig {};
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let config: Config = from_json(&res).unwrap();
        assert_eq!(
            config,
            Config {
                owner: Addr::unchecked("owner"),
                cw721_address: Some(Addr::unchecked(NFT_CONTRACT_ADDR)),
                max_tokens: msg.max_tokens,
                unit_price: msg.unit_price,
                name: msg.name,
                symbol: msg.symbol,
                token_uri: msg.token_uri,
                extension: None,
                unused_token_id: 0,
            }
        );
    }
}
