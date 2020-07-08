use cosmwasm_std::{
    coin, to_binary, Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, InitResponse,
    Querier, StdError, StdResult, Storage,
};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, Item, State, USCRT_DENOM};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut items = Vec::<Item>::new();
    // Init msg.item_count items
    for i in 0..msg.items_count {
        items.push(Item {
            id: i,
            value: coin(1, USCRT_DENOM),
            owner: env.message.sender.clone(),
        });
    }

    // The golden item
    if msg.golden != 0 {
        items[(msg.golden - 1) as usize].value = coin(100, USCRT_DENOM);
    } else {
        // TODO random placement
        items[0].value = coin(100, USCRT_DENOM);
    }

    // Create state
    let state = State {
        items,
        contract_owner: env.message.sender.clone(),
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
        HandleMsg::Increment {} => try_increment(deps, env),
        HandleMsg::Reset { count } => try_reset(deps, env, count),
    }
}

pub fn try_increment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    // config(&mut deps.storage).update(|mut state| {
    //     state.count += 1;
    //     Ok(state)
    // })?;

    Ok(HandleResponse::default())
}

pub fn try_reset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    count: i32,
) -> StdResult<HandleResponse> {
    // config(&mut deps.storage).update(|mut state| {
    //     if env.message.sender != state.owner {
    //         return Err(StdError::Unauthorized { backtrace: None });
    //     }
    //     state.count = count;
    //     Ok(state)
    // })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(CountResponse { count: 0 })
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
fn balance_of(owner: CanonicalAddr) -> u32 {
    unimplemented!()
}

/// @notice Find the owner of an NFT
/// @dev NFTs assigned to zero address are considered invalid, and queries
///  about them do throw.
/// @param _tokenId The identifier for an NFT
/// @return The address of the owner of the NFT
fn owner_of(token_id: u32) -> CanonicalAddr {
    unimplemented!()
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
fn safe_transfer_from_with_data(
    from: CanonicalAddr,
    to: CanonicalAddr,
    token_id: u32,
    data: &[u8],
) {
    unimplemented!()
}

/// @notice Transfers the ownership of an NFT from one address to another address
/// @dev This works identically to the other function with an extra data parameter,
///  except this function just sets data to "".
/// @param _from The current owner of the NFT
/// @param _to The new owner
/// @param _tokenId The NFT to transfer
fn safe_transfer_from(from: CanonicalAddr, to: CanonicalAddr, token_id: u32) {
    unimplemented!()
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
fn transfer_from(from: CanonicalAddr, to: CanonicalAddr, token_id: u32) {
    unimplemented!()
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
