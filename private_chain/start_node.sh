#!/bin/bash
# input: $1 account name: alice/bob
# $2 index: starting with 0, 0 is seed node 
# $3 peerId of seed node, used to bootstrap, not needed for seed node
# e.g 1: ./start_node.sh alice 0
# e.g 2: ./start_node.sh node1 1 <peerId>

file_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$file_path"

#replace it to your program location, e.g. ../target/release/e2-chain
program=../target/debug/e2-chain

port=$((30333+$2))
ws_port=$((9944+$2))
rpc_port=$((9933+$2))

# start at genesis block with fresh database
#rm -r /tmp/$1 2&>/dev/null || true

if [ ! -z "$3" ]
then 
    echo "starting node $1 ..."
    echo "boot node :  $3 "
    $program  --base-path /tmp/$1 \
    --chain customSpecRaw.json \
    --port $port \
    --ws-port $ws_port \
    --rpc-port $rpc_port \
    --validator \
    --rpc-methods=Unsafe \
    --rpc-cors all \
    --name $1 \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/$3
else
    echo "starting seed node $1 ..."
    $program  --base-path /tmp/$1 \
    --chain customSpecRaw.json \
    --port $port \
    --ws-port $ws_port \
    --rpc-port $rpc_port \
    --validator \
    --rpc-methods=Unsafe \
    --rpc-cors all \
    --name $1
fi
