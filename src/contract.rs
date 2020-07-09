use cosmwasm_std::{
    coin, log, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage,
    Uint128,
};
use lazy_static::lazy_static;

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, Item, State, USCRT_DENOM};

lazy_static! {
    static ref ZERO_ADDRESS: CanonicalAddr = CanonicalAddr(Binary(vec![0; 8]));
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // Init msg.item_count items
    let mut items = Vec::<Item>::new();
    for i in 0..msg.items_count {
        items.push(Item {
            id: i,
            value: coin(1, USCRT_DENOM.clone()),
            owner: env.contract.address.clone(),
            approved: Vec::<CanonicalAddr>::new(),
        });
    }

    // if env.message.sent_funds[0].amount.u128() <= msg.items_count as u128 {
    //     return Err(throw_gen_err(format!(
    //         "You sent only {:?} tokens for {:?} items.",
    //         env.message.sent_funds[0].amount, msg.items_count
    //     )));
    // }

    // Building the winning representation as a coin
    let winning_prize = Coin {
        denom: USCRT_DENOM.to_string(),
        amount: env.message.sent_funds[0].amount,
    };

    items[msg.golden as usize].value = winning_prize.clone();

    // Create state
    let state = State {
        items,
        contract_owner: env.message.sender.clone(),
        winning_prize: winning_prize.clone(),
    };

    // Save to state
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SafeTransferFrom { from, to, token_id } => {
            safe_transfer_from(deps, env, &from, &to, token_id)
        }
        HandleMsg::BuyTicket { token_id } => buy_ticket(deps, env, token_id),
        HandleMsg::EndLottery {} => end_lottery(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::BalanceOf { owner } => to_binary(&balance_of(deps, &owner)),
        QueryMsg::OwnerOf { token_id } => to_binary(&owner_of(deps, token_id)),
    }
}

// ERC-721 interface

/// @dev This emits when ownership of any NFT changes by any mechanism.
///  This event emits when NFTs are created (`from` == 0) and destroyed
///  (`to` == 0). Exception: during contract creation, any number of NFTs
///  may be created and assigned without emitting Transfer. At the time of
///  any transfer, the approved address for that NFT (if any) is reset to none.
// fn transfer(from: CanonicalAddr, to: CanonicalAddr, token_id: u32) {
//     unimplemented!()
// }

/// @dev This emits when the approved address for an NFT is changed or
///  reaffirmed. The zero address indicates there is no approved address.
///  When a Transfer event emits, this also indicates that the approved
///  address for that NFT (if any) is reset to none.
// event Approval(address indexed _owner, address indexed _approved, uint256 indexed _tokenId);

/// @dev This emits when an operator is enabled or disabled for an owner.
///  The operator can manage all NFTs of the owner.
// event ApprovalForAll(address indexed _owner, address indexed _operator, bool _approved);

/// @notice Count all NFTs assigned to an owner
/// @dev NFTs assigned to the zero address are considered invalid, and this
///  function throws for queries about the zero address.
/// @param _owner An address for whom to query the balance
/// @return The number of NFTs owned by `_owner`, possibly zero
fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: &HumanAddr,
) -> StdResult<u32> {
    let owner_addr_raw = deps.api.canonical_address(&owner)?;

    if owner_addr_raw == *ZERO_ADDRESS {
        return Err(throw_gen_err("Can't query the zero address!".to_string()));
    }

    let state = config_read(&deps.storage).load()?;
    let mut count = 0;

    for item in state.items {
        if item.owner == owner_addr_raw {
            count = count + 1;
        }
    }

    Ok(count)
}

/// @notice Find the owner of an NFT
/// @dev NFTs assigned to zero address are considered invalid, and queries
///  about them do throw.
/// @param _tokenId The identifier for an NFT
/// @return The address of the owner of the NFT
fn owner_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: u32,
) -> StdResult<HumanAddr> {
    let state = config_read(&deps.storage).load()?;

    // Check to not go out of bounds
    if !is_token_id_valid(token_id, &state) {
        return Err(throw_gen_err(format!(
            "Item {:?} does not exist!",
            token_id
        )));
    }

    let owner_addr_raw = state.items[token_id as usize].owner.clone();

    // Check if item has been redeemed
    if owner_addr_raw == *ZERO_ADDRESS {
        return Err(throw_gen_err(format!(
            "Item {:?} has been redeemed already!",
            token_id
        )));
    }

    deps.api.human_address(&owner_addr_raw)
}

/// @notice Transfers the ownership of an NFT from one address to another address
/// @dev Throws unless `msg.sender` is the current owner, an authorized
///  operator, or the approved address for this NFT. Throws if `_from` is
///  not the current owner. Throws if `_to` is the zero address. Throws if
///  `_tokenId` is not a valid NFT. When transfer is complete, this function
///  checks if `_to` is a smart contract (code size > 0). If so, it calls
///  `onERC721Received` on `_to` and throws if the return value is not
///  `bytes4(keccak256("onERC721Received(address,address,uint256,bytes)"))`.
/// @param _from The current owner of the NFT
/// @param _to The new owner
/// @param _tokenId The NFT to transfer
/// @param data Additional data with no specified format, sent in call to `_to`
fn safe_transfer_from_with_data<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: &HumanAddr,
    to: &HumanAddr,
    token_id: u32,
    data: Vec<u8>,
) -> StdResult<HandleResponse> {
    // Canonicalize addrs
    let from_addr_raw = deps.api.canonical_address(from)?;
    let to_addr_raw = deps.api.canonical_address(to)?;

    // Throw if `to` is the zero address
    if to_addr_raw == *ZERO_ADDRESS {
        return Err(throw_gen_err(format!(
            "Can't burn Items with `safe_transfer_from` function. To burn an Item, use the unsafe `transfer_ftom`"        )));
    }

    // Get item from state
    let state = config_read(&mut deps.storage).load()?;
    let item = state.items[token_id as usize].clone();

    // Check if owner or approved
    if !is_owner_or_approved(&item, &env.message.sender) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // From has to be the owner
    if from_addr_raw != item.owner {
        return Err(throw_gen_err(format!(
            "{:?} is not the owner of {:?} Item!",
            from, token_id
        )));
    }

    if !is_token_id_valid(token_id, &state) {
        return Err(throw_gen_err(format!(
            "Item {:?} does not exist!",
            token_id
        )));
    }

    // Perform transfer
    match perform_transfer(deps, &to_addr_raw, token_id) {
        Ok(_) => Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(Binary(data.to_vec())),
        }),
        Err(e) => {
            return Err(throw_gen_err(format!(
                "Error transferring Item {:?}: {:?}",
                token_id, e
            )))
        }
    }
}

/// @notice Transfers the ownership of an NFT from one address to another address
/// @dev This works identically to the other function with an extra data parameter,
///  except this function just sets data to "".
/// @param _from The current owner of the NFT
/// @param _to The new owner
/// @param _tokenId The NFT to transfer
fn safe_transfer_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: &HumanAddr,
    to: &HumanAddr,
    token_id: u32,
) -> StdResult<HandleResponse> {
    safe_transfer_from_with_data(deps, env, from, to, token_id, vec![])
}

/// @notice Transfer ownership of an NFT -- THE CALLER IS RESPONSIBLE
///  TO CONFIRM THAT `_to` IS CAPABLE OF RECEIVING NFTS OR ELSE
///  THEY MAY BE PERMANENTLY LOST
/// @dev Throws unless `msg.sender` is the current owner, an authorized
///  operator, or the approved address for this NFT. Throws if `_from` is
///  not the current owner. Throws if `_to` is the zero address. Throws if
///  `_tokenId` is not a valid NFT.
/// @param _from The current owner of the NFT
/// @param _to The new owner
/// @param _tokenId The NFT to transfer
fn transfer_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: &HumanAddr,
    to: &HumanAddr,
    token_id: u32,
) -> StdResult<HandleResponse> {
    // Currently it's the same implementation
    safe_transfer_from(deps, env, from, to, token_id)
}

/// @notice Change or reaffirm the approved address for an NFT
/// @dev The zero address indicates there is no approved address.
///  Throws unless `msg.sender` is the current NFT owner, or an authorized
///  operator of the current owner.
/// @param _approved The new approved NFT controller
/// @param _tokenId The NFT to approve
fn approve(approved: CanonicalAddr, token_id: u32) {
    unimplemented!()
}

/// @notice Enable or disable approval for a third party ("operator") to manage
///  all of `msg.sender`'s assets
/// @dev Emits the ApprovalForAll event. The contract MUST allow
///  multiple operators per owner.
/// @param _operator Address to add to the set of authorized operators
/// @param _approved True if the operator is approved, false to revoke approval
fn set_approval_for_all(operator: CanonicalAddr, approved: bool) {
    unimplemented!()
}

/// @notice Get the approved address for a single NFT
/// @dev Throws if `_tokenId` is not a valid NFT.
/// @param _tokenId The NFT to find the approved address for
/// @return The approved address for this NFT, or the zero address if there is none
fn get_approved(token_id: u32) -> CanonicalAddr {
    unimplemented!()
}

/// @notice Query if an address is an authorized operator for another address
/// @param _owner The address that owns the NFTs
/// @param _operator The address that acts on behalf of the owner
/// @return True if `_operator` is an approved operator for `_owner`, false otherwise
fn is_approved_for_all(owner: CanonicalAddr, operator: CanonicalAddr) -> bool {
    unimplemented!()
}

fn throw_gen_err(msg: String) -> StdError {
    StdError::GenericErr {
        msg,
        backtrace: None,
    }
}

fn is_owner_or_approved(item: &Item, addr: &CanonicalAddr) -> bool {
    addr == &item.owner || item.approved.clone().iter().any(|i| i == addr)
}

fn is_token_id_valid(token_id: u32, state: &State) -> bool {
    (token_id as usize) < state.items.len()
}

fn perform_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    to: &CanonicalAddr,
    token_id: u32,
) -> StdResult<State> {
    config(&mut deps.storage).update(|mut state| {
        state.items[token_id as usize].owner = to.clone();
        Ok(state)
    })
}

fn buy_ticket<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token_id: u32,
) -> StdResult<HandleResponse> {
    let sent_funds: Coin = env.message.sent_funds[0].clone();

    // TODO min entry price in state
    if sent_funds.amount.u128() < 1 {
        return Err(throw_gen_err(format!(
            "You sent {:?} funds, it is not enough!",
            sent_funds.amount
        )));
    }

    // let contract_addr_human = deps.api.human_address(&env.contract.address)?;
    // let sender_addr_human = deps.api.human_address(&env.message.sender)?;

    // Transfer coin to buyer
    perform_transfer(deps, &env.message.sender, token_id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn end_lottery<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // TODO Check if contract has expired

    let state = config(&mut deps.storage).load()?;
    let mut messages: Vec<CosmosMsg> = vec![];

    for item in state.items {
        let contract_addr: HumanAddr = deps.api.human_address(&env.contract.address)?;
        let to: HumanAddr = deps.api.human_address(&item.owner)?;

        if to == contract_addr {
            // to = deps.api.human_address(&state.contract_owner)?;
            continue;
        }

        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: contract_addr.clone(),
            to_address: to.clone(),
            amount: vec![item.value],
        }))

        // TODO Transfer item to zero address
    }

    // TODO if anything left in the deposit, return to contract owner

    // TODO mark lottery as ended

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
    })
}
