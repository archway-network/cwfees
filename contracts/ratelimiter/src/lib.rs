use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{coin, entry_point, Addr, Coin, Coins, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, CoinsError};
use cw_storage_plus::{Bound, Item, Map};
use cwfees::{CwGrant, SudoMsg};
use thiserror::Error;
use crate::ContractError::Unauthorized;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("{0} is not authorized to use the grants")]
    Unauthorized(String),
    #[error("only one message allowed")]
    OnlyOneMessage,
    #[error("limits reached")]
    LimitsReached,
    #[error("coins error: {0}")]
    CoinsError(#[from] CoinsError)
}

pub struct InstantiateMsg {
    pub allow_list_and_limits: Vec<(String, Coins)>,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const LIMITS: Map<Addr, Vec<Coin>> = Map::new("limits"); // tracks hourly limits.

pub const USAGE: Map<(Addr, u64), Vec<Coin>> = Map::new("usage"); // tracks usage, as (Timestamp_when_used, User)=>amount

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    for (addr, limit) in msg.allow_list_and_limits {
        let addr = deps.api.addr_validate(&addr)?;
        LIMITS.save(deps.storage, addr, &limit.into())?;
    }

    // register the contract as granter in the archway state machine.
    Ok(Response::default().add_message(cwfees::new_register_as_granter_msg(env.contract.address)))
}

// TODO: add execute here using OWNER to update rate limits etc.

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    return match msg {
        SudoMsg::CwGrant(request) => process_grant(deps, env, request),
        _ => Err(Unauthorized("unknown".to_string()))
    };
}
fn process_grant(deps: DepsMut, env: Env, req: CwGrant) -> Result<Response, ContractError> {
    // first we fetch the limits, so if one of the senders is not present in the limits then we can simply
    // assume it's not authorized to use the grants, for the sake of the simplicity of the example
    // we assume our granting contract can only have one msg.
    if req.msgs.len() > 1 {
        return Err(ContractError::OnlyOneMessage);
    }

    let sender = deps.api.addr_validate(&req.msgs[0].sender)?;

    let limits = LIMITS
        .may_load(deps.storage, sender.clone())?
        .ok_or_else(|| ContractError::Unauthorized(sender.to_string()))?;

    // now we count the usage in the last hour!
    let last_hour_ts = env.block.time.minus_hours(1).seconds();

    let coins_used = USAGE
        .prefix(sender.clone()) // we prefix over the requester of funds.
        .range(
            deps.storage,
            Some(Bound::inclusive(last_hour_ts)), // we scan from last hour
            Some(Bound::inclusive(env.block.time.seconds())), // until now
            Ascending,
        )
        .map(|r| r.map(|r| r.1))
        .collect::<StdResult<Vec<Vec<Coin>>>>()?;

    check_limits(limits, coins_used, req.fee_requested.clone())?;

    // store usage
    USAGE.save(
        deps.storage,
        (sender, env.block.time.seconds()),
        &req.fee_requested,
    )?;

    Ok(Response::default())
}

fn check_limits(limits: Vec<Coin>, used: Vec<Vec<Coin>>, requested: Vec<Coin>) -> Result<(), ContractError> {
    let mut limits = Coins::try_from(limits)?;
    for coin in used.into_iter().flatten().chain(requested) {
        if let Err(_) = limits.sub(coin) {
            return Err(ContractError::LimitsReached)
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn builds() {}
}
