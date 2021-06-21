#### "subkey" examples

```
\$ subkey generate
Secret phrase `spy stereo denial truck slot typical blush material can resemble cool banana` is account:
Secret seed: 0xe3e6916f53e0b669e184c4a8a8606a07f0490aaae72fe3b63e6d35906d5932a2
Public key (hex): 0x7e3407056f176e1c0387cdec18af27f401c553f65956b024c23a4ca6d17ca27c
Account ID: 0x7e3407056f176e1c0387cdec18af27f401c553f65956b024c23a4ca6d17ca27c
SS58 Address: 5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC
```

```
\$ subkey inspect --scheme ed25519
URI:
Secret phrase `spy stereo denial truck slot typical blush material can resemble cool banana` is account:
Secret seed: 0xe3e6916f53e0b669e184c4a8a8606a07f0490aaae72fe3b63e6d35906d5932a2
Public key (hex): 0xa1787c2f2286839cc88c57c47976b40627692f60bdebfae38d8d71157e5b7c42
Account ID: 0xa1787c2f2286839cc88c57c47976b40627692f60bdebfae38d8d71157e5b7c42
SS58 Address: 5FiRMj4QcAv2dwAm2XdtL2vmanvngVawo3YAGmURW1JTRkmK
```

```
\$ subkey generate
Secret phrase `mandate zoo couch coin seat goose devote physical recipe lift intact receive` is account:
Secret seed: 0x55b2e3194ed5e3cd900e80b601abb85ebbc093d5632cd2bfee29cfd9e099f976
Public key (hex): 0x6c96bdc62cb26d13e149ebb0ddf6d1f71c7b975d4dd1e31875d72125c461e20a
Account ID: 0x6c96bdc62cb26d13e149ebb0ddf6d1f71c7b975d4dd1e31875d72125c461e20a
SS58 Address: 5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa
```

```
\$ subkey inspect --scheme ed25519
URI:
Secret phrase `mandate zoo couch coin seat goose devote physical recipe lift intact receive` is account:
Secret seed: 0x55b2e3194ed5e3cd900e80b601abb85ebbc093d5632cd2bfee29cfd9e099f976
Public key (hex): 0x1ece27096ef9d6a1f5908d8dc8fc6c60b446b42d04d20ddcaa4f13a7bfe4223f
Account ID: 0x1ece27096ef9d6a1f5908d8dc8fc6c60b446b42d04d20ddcaa4f13a7bfe4223f
SS58 Address: 5Cm6XutwYAMd3wZvJJPwaXwMD4zT6qKNw4U5zazAd5Q3P9Bb
```

#### add new keys

Follow the instructions of "Start a private network" at https://learnblockchain.cn/docs/substrate/docs/tutorials/start-a-private-network/customchain/
and replace with json configuration file with the keys generated above

Babe:

-   Basically follow the instructions of "Start a private network" at https://learnblockchain.cn/docs/substrate/docs/tutorials/start-a-private-network/customchain/
    But must use substrate instead of node-template.
    For example, to generate chain spec json file:
    ./target/release/substrate build-spec --disable-default-bootnode --chain local > customSpec.json
-   Need to generate two more keys based on the secret phrases as above by concatenating "//stash" in the end.

```
\$ subkey inspect
URI:
Secret Key URI `spy stereo denial truck slot typical blush material can resemble cool banana//stash` is account:
Secret seed: 0xe33f1bd24cc6c49ea692cc43957d08978f50a7dd18552e4a85fbac1695c8dc38
Public key (hex): 0xfc71ed3de22b7a75725d90edb831a6b2d34a37578a2460ba976ee76e7bb5b60f
Account ID: 0xfc71ed3de22b7a75725d90edb831a6b2d34a37578a2460ba976ee76e7bb5b60f
SS58 Address: 5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd
```

```
\$ subkey inspect
URI:
Secret Key URI `mandate zoo couch coin seat goose devote physical recipe lift intact receive//stash` is account:
Secret seed: 0xf017e07a3a7227a78adf83a32879bef4585a2b128ce0e756e967cb232dc3bba8
Public key (hex): 0xa210a609032842e199edcca63cd2fa9312474e0a8f45013f390a71a9885b2863
Account ID: 0xa210a609032842e199edcca63cd2fa9312474e0a8f45013f390a71a9885b2863
SS58 Address: 5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53
```

I've committed two json configuration files for your reference: one is customSpec.json which is generated following the logic of "Start a private network",
another one "chain_spec.json" is created based on customSpec.json with keys replaced.

The following configurations are updated in the chain spec json file:

```
"palletBalances": {
"balances": [
[
"5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
1000000000000000000000
],
[
"5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa", <= sr25519
1000000000000000000000
],
[
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
1000000000000000000000
],
[
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53", <= special sr25519
1000000000000000000000
],
[
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
10000000000000000
],
[
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53", <= special sr25519
10000000000000000
]
]
},
```

```
"palletStaking": {
"historyDepth": 84,
"validatorCount": 4,
"minimumValidatorCount": 2,
"invulnerables": [
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53" <= special sr25519
],
"forceEra": "NotForcing",
"slashRewardFraction": 100000000,
"canceledPayout": 0,
"stakers": [
[
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
"5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
10000000000000000,
"Validator"
],
[
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53", <= special sr25519
"5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa", <= sr25519
10000000000000000,
"Validator"
]
]
},
```

```
"palletSession": {
"keys": [
[
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
"5HmhmTq3uCBneXLpq9YcF8F3ZoVrSW5wCYkCKHB1DeEmRnNd", <= special sr25519
{
"grandpa": "5FiRMj4QcAv2dwAm2XdtL2vmanvngVawo3YAGmURW1JTRkmK", <= ed25519
"babe": "5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
"im_online": "5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
"authority_discovery": "5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC" <= sr25519
}
],
[
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53", <= special sr25519
"5FjCZSa9h4Bv8RXQ49j8xCZaQo6vZqLYomDyaUhmKLmE4B53", <= special sr25519
{
"grandpa": "5Cm6XutwYAMd3wZvJJPwaXwMD4zT6qKNw4U5zazAd5Q3P9Bb", <= ed25519
"babe": "5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa", <= sr25519
"im_online": "5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa", <= sr25519
"authority_discovery": "5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa" <= sr25519
}
]
]
},
```

```
"palletCollectiveInstance2": {
"phantom": null,
"members": [
"5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
"5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa" <= sr25519
]
},
```

```
"palletElectionsPhragmen": {
"members": [
[
"5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
10000000000000000
],
[
"5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa", <= sr25519
10000000000000000
]
]
},
```

```
"palletSudo": {
"key": "5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC" <= sr25519
},

```

```
"palletSociety": {
"pot": 0,
"maxMembers": 999,
"members": [
"5EvBM2DomjSxL9WantCMCzc6MeqS6zD84MThNJ86DXbdgUmC", <= sr25519
"5EX5oq8waRkvMchq4BdyoPJQ9cZEGvRus6bv2m3QCeXPcTaa" <= sr25519
]
},

```

In the key import step, we need import "babe" key instead of import "aura".
Using Polkadot-JS Apps UI for key import is simple.

### Start a private testnet

Config and key generation

```
# In private_chain folder:
# get template from "local" config
../target/debug/deeper-chain build-spec --disable-default-bootnode --chain local > first.json
# add new key into genesis config
./chain_spec_gen.py -i first.json -o second.json
# or import existing key into genesis config
./chain_spec_gen.py -i first.json -o second.json -k <loc_to_key_file>
# create raw config
../target/debug/deeper-chain build-spec --chain=second.json --raw --disable-default-bootnode > customSpecRaw.json
```

Start a private network

```
# In private_chain folder:
./start_node.sh alice 0  # bootstrap node
./start_node.sh bob 1 <peerID> # peerID copied from bootstrap node
./start_node.sh chao 2 <peerID>
# insert babe and grandpa keys:
./insert_key.sh 9933 alice.json alice_gran.json
./insert_key.sh 9934 bob.json bob_gran.json
./insert_key.sh 9935 chao.json chao_gran.json
```
