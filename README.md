# CW721 Contract Sample

This repository contains a sample implementation of a CW721 contract, a standardized way to represent and manage non-fungible 
tokens (NFTs) on a CosmWasm-enabled blockchain. CW721 is an extension of the ERC721 standard, popularized by the Ethereum blockchain,
allowing for the creation, management, and transfer of NFTs.

## Features
- Minting: Allows the creation of new NFTs.
- Burning: NFTs can be destroyed, reducing the total supply.
- Transfer: Users can send NFTs to other addresses.
- Approve/Revoke: Provides the ability to approve other addresses to transfers NFTs on behalf of the user.
- Querying: Users can query balances, total supply, allowances, and more.


## Requirements
Rust and Cargo for building the contract.

## Getting Started

1. Clone the Repository

```bash
git clone https://github.com/MANTRA-Finance/cw721_contract_sample.git
cd cw721_contract_sample
```

2. Build the Contract

```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```

Or if you are on MacOS:

```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer-arm64:0.16.0
```

3. Deploy the Contract

```bash
mantrachaind tx wasm store artifacts/moni.wasm --from <wallet> --node https://rpc.hongbai.mantrachain.io:443 --chain-id mantra-hongbai-1 --gas-prices 0.35uom --gas auto --gas-adjustment 1.4 -y --output json
```
4. Instantiate the Contract

```bash
MSG='{
  "owner": "", // add your wallet
  "max_tokens": 10000,
  "unit_price": {
    "denom": "uom",
    "amount": "1000"
  },
  "name": "Mantra NFT",
  "symbol": "MANTRANFT",
  "token_code_id": 507, // the code id of a cw721_base contract
  "token_uri": "https://ipfs.io/ipfs/QmZ"
}'

mantrachaind tx wasm instantiate <code_id> "$MSG" --from <wallet> --node https://rpc.hongbai.mantrachain.io:443 --chain-id mantra-hongbai-1 --label "MANTRACW721" --no-admin --gas-prices 0.35uom --gas auto --gas-adjustment 1.4 -y --output json

```

## Refer to MANTRA Chain Docs for learning more

https://docs.mantrachain.io/
