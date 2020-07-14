# Secret Lottery

An example for a secret contract implementation on [Secret Network](https://github.com/enigmampc/SecretNetwork) using our custom [CosmWasm](https://github.com/CosmWasm/cosmwasm) module.

This is in fact a partial implementation of [ERC-721 (NFT)](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-721.md).

## Description
This is a simple lottery game. The 'Lottery Host' will deploy an instance of this contract, setting an initial prize fund (say, 100SCRT), how many tickets to create (say, 150 tickets) and which ticket is the winning one. Then, anyone can buy a lottery ticket (for, say, 1SCRT), by paying to the lottery fund. One can also trade their ticket, set operators, etc..

When the lottery is ended, every participant gets the underlying value of their ticket (e.g. winning ticker = 100SCRT, normal ticket = 0SCRT). The host will get the remaining amount in the lottery fund. A participant can win a prize of 100SCRT, while the lottery host can earn 50SCRT (given that all tickets are bought).

## Disclaimer
This is only a usage example, and does not imply on how to correctly and safely use or write `Secret Contracts`. You should always make sure to read and understand `Secret Contract` API's disclaimers and limitations before deploying a contract in production!

## Usage
Store the contract on-chain:
```bash
secretcli tx compute store contract.wasm.gz --from account
```

Instantiate contract:
```bash
secretcli tx compute instantiate 1 '{ "name":"secret_lottery", "tickets_count":100, "winning": 97 }' --label secret-lottery --from account --amount 100000000uscrt # = 100SCRT
```

Buy a ticket:
```bash
secretcli tx compute execute <contract-address> '{ "buy_ticket": { "ticket_id": 1 }}' --from account --amount 1000000uscrt # = 1SCRT
```

End lottery:
```bash
secretcli tx compute execute <contract-address> '{ "end_lottery": {} }' --from account
```

For more details, check out the [messages module](https://github.com/toml01/SecretLottery/blob/master/src/msg.rs).