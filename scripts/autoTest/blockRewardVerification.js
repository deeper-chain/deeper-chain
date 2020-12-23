const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const ObjectsToCsv = require('objects-to-csv');

const serverHost = "wss://10.168.98.1:443";


async function getBlockInfo() {
    const wsProvider = new WsProvider(serverHost);

    const api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            TokenBalance: "u64",
            Timestamp: "Moment",
            Node: {
                account_id: "AccountId",
                ipv4: "Vec<u8>",
                country: "u16",
            },
            ChannelOf: {
                sender: "AccountId",
                receiver: "AccountId",
                nonce: "u64",
                opened: "Timestamp",
                expiration: "Timestamp",
            },
            CreditDelegateInfo: {
                delegator: "AccountId",
                score: "u64",
                validators: "Vec<AccountId>"
            }
        },
    });
    const keyring = new Keyring({ type: "sr25519" });

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    var EraRewardList = new Array();
    var index = 0;

    for (var i = 73; i < 167666; i += 72) {
        // returns Hash
        var blocknumber = i;
        const blockHash = await api.rpc.chain.getBlockHash(blocknumber);
        // returns SignedBlock
        const signedBlock = await api.rpc.chain.getBlock(blockHash);

        // the hash for the block, always via header (Hash -> toHex()) - will be
        // the same as blockHash above (also available on any header retrieved,
        // subscription or once-off)
        console.log(signedBlock.block.header.hash.toHex());

        const allRecords = await api.query.system.events.at(signedBlock.block.header.hash);

        // Loop through the Vec<EventRecord>
        allRecords.forEach((record) => {
            // Extract the phase, event and the event types
            const { event, phase } = record;
            const types = event.typeDef;

            if (event.section == "staking" && event.method == "EraPayout") {
                // Show what we are busy with
                console.log(`\t${event.section}:${event.method}:: (phase=${phase.toString()})`);
                //console.log(`\t\t${event.meta.documentation.toString()}`);

                var reward = {"EraIndex":event.data[0].toString(), "Balance":event.data[1].toString()}
                EraRewardList[index++] = reward;
                // Loop through each of the parameters, displaying the type and data
                event.data.forEach((data, index) => {
                    console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
                });
            }
        });
    }

    const csv = new ObjectsToCsv(EraRewardList);
    await csv.toDisk("blockRewardInfo.csv");
}

getBlockInfo().catch(console.error).finally(() => process.exit());