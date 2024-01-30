use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;
use cosmos_sdk_proto::prost;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, Addr, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError};
use cw_storage_plus::{Item, Map};
use thiserror::Error;
use cwfees::{CwGrant, SudoMsg};

pub const OWNER: Item<Addr> = Item::new("owner");

pub const ALLOWED_CONTRACT: Item<Addr> = Item::new("allowed_contract");

pub const ALLOWED_ADDRESSES: Map<Addr, Empty> = Map::new("allowed_addresses");

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("unauthorized {0}")]
    Unauthorized(String),
    #[error("message is not in the allow list {0}")]
    DisallowedMessage(String),
    #[error("not allowed to spend fees on contract {0}")]
    DisallowedContract(String),
    #[error("decode error")]
    DecodeError(#[from] prost::DecodeError)
}

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_addresses: Vec<String>,
    pub allowed_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RemoveAllowance(String),
    AddAllowance(String),
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    for addr in msg.allowed_addresses {
        let addr = deps.api.addr_validate(&addr)?;
        ALLOWED_ADDRESSES.save(deps.storage, addr, &Empty::default())?;
    }

    ALLOWED_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.allowed_contract)?,
    )?;

    OWNER.save(deps.storage, &info.sender)?;

    Ok(Response::default().add_message(cwfees::new_register_as_granter_msg(env.contract.address)))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::Unauthorized(info.sender.into_string()))
    }

    match msg {
        ExecuteMsg::RemoveAllowance(addr) => {
            ALLOWED_ADDRESSES.remove(deps.storage, deps.api.addr_validate(&addr)?)
        }
        ExecuteMsg::AddAllowance(addr) => {
            ALLOWED_ADDRESSES.save(deps.storage, deps.api.addr_validate(&addr)?, &Empty::default())?
        }
    };

    return Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    return match msg {
        SudoMsg::CwGrant(grant) => process_grant(deps, grant),
        _ => Err(ContractError::Unauthorized("unknown method".to_string()))
    }
}

fn process_grant(deps: DepsMut, grant: CwGrant) -> Result<Response, ContractError> {
    let allowed_contract = ALLOWED_CONTRACT.load(deps.storage)?;
    for msg in grant.msgs {
        // we check if all the senders are in the allow list
        let addr = deps.api.addr_validate(&msg.sender)?;
        if !ALLOWED_ADDRESSES.has(deps.storage, addr) {
            return Err(ContractError::Unauthorized(msg.sender.clone()))
        }

        // we check the message type url
        if msg.type_url != "" {
            return Err(ContractError::DisallowedMessage(msg.type_url))
        }

        // we check inside the message if the user is trying to spend fees
        // in some other contract.
        let msg: MsgExecuteContract = msg.try_into_proto()?;
        if msg.contract != allowed_contract.to_string() {
            return Err(ContractError::DisallowedContract(msg.contract))
        }
    }

    Ok(Response::default())
}


#[cfg(test)]
mod test {
    #[test]
    fn build() {}
}