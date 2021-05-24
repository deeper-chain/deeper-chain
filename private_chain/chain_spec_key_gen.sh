../target/debug/deeper-chain build-spec --disable-default-bootnode --chain local > first.json
echo "*****gen customSpec.json with Alic and Bob"
./chain_spec_gen.py -i first.json -o second.json -n 5
echo "****add chain configuration and the third account chao to customSpec.json"
../target/debug/deeper-chain build-spec --chain=second.json --raw --disable-default-bootnode > customSpecRaw.json
echo "****gen customSpecRaw.json"