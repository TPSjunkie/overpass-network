#!/bin/bash

# Deploy the channels to the Overpass Devnet 2 testnet
./target/release/overpass-channels deploy-channels  --config ./config/config.toml  --chain-id=overpass-devnet-2 --keyring-backend=file --keyring-dir=/Users/cryptskii/keys --from=overpass --gas=auto --fees=1000uoverpass --broadcast-mode=block --yes --gas-prices=1000uoverpass --gas-adjustment=1.5 --node=https://rpc.testnet.overpass.wtf:443 --output ./output/deploy_channels.json  --chain-id=overpass-devnet-2 --keyring-backend=file --keyring-dir=/Users/cryptskii/keys --from=overpass --gas=auto --fees=1000uoverpass --broadcast-mode=block --yes --gas-prices=1000uoverpass --gas-adjustment=1.5 --node=https://rpc.testnet.overpass.wtf:443 --output ./output/deploy_channels.json 
