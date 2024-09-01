#!/bin/bash

mkdir -p ./target/tmp 
cd ./target/tmp

if [ -z "$1" ]; then
    echo "Please provide node version to download."
    exit 1
fi

wget https://github.com/paritytech/substrate-contracts-node/releases/download/$1/substrate-contracts-node-linux.tar.gz -O zipped-node.tar.gz
NODE_DIR=$(pwd)

tar -xzf zipped-node.tar.gz 
mv artifacts/substrate-contracts-node-linux/substrate-contracts-node ./

# set env if not set already
if grep -Fxq PATH=$NODE_DIR:\$PATH ~/.bashrc 
then
echo export PATH="$NODE_DIR:\$PATH" >> ~/.bashrc
fi
source ~/.bashrc