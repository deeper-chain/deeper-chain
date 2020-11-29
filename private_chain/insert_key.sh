# Submit a new key via RPC, connect to where your `rpc-port` is listening
# ./insert_key <port> <file1> <file2>
# port: 9933, 9934, ....
curl http://localhost:$1 -H "Content-Type:application/json;charset=utf-8" -d @$2
curl http://localhost:$1 -H "Content-Type:application/json;charset=utf-8" -d @$3

# insert node: ./insert_key <node_name> <secret_seed>
# e.g: ./insert_key alice <secret_seed>
#subkey insert --suri $2 --base-path /tmp/$1 --key-type babe 
#subkey insert --suri $2 --scheme ed25519 --base-path /tmp/$1 --key-type gran
