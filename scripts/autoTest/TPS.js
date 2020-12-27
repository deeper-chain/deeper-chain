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

    api.query.timestamp
    api.derive.chain

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    var blockList = new Array();
    var index = 0;
    var lastBlockTime = 0;
    var lastFinalityBlockNumber = 0;
    for (var i = 1083; i < 1601; i++) {
        // returns Hash
        var blocknumber = i;
        const blockHash = await api.rpc.chain.getBlockHash(blocknumber);
        // returns SignedBlock
        const signedBlock = await api.rpc.chain.getBlock(blockHash);

        var extrinsics = signedBlock.block.extrinsics;
        var extrinsic_number = extrinsics.length;
        if (extrinsic_number == 1) {
            var { isSigned, meta, method: { args, method, section } } = extrinsics[0];
            var blocktime = args[0];
            console.log(`blocknumber = ${blocknumber}, blocktime = ${blocktime}, extrinsic number = ${extrinsic_number}, finalityBlockNumber = ${lastFinalityBlockNumber}`);
            if (lastBlockTime == 0) {
                lastBlockTime = blocktime;
                var blockInfo = { "blockNumber": blocknumber, "blockDeltTime": 0, "extrinsicNumber": extrinsic_number, "TPS": 0 , "finalityDelayBlocks": blocknumber - lastFinalityBlockNumber};
                blockList[index++] = blockInfo;
            } else {
                var blockInfo = { "blockNumber": blocknumber, "blockDeltTime": blocktime - lastBlockTime, "extrinsicNumber": extrinsic_number,
                 "TPS": ((extrinsic_number * 1000) / (blocktime - lastBlockTime)).toFixed(2), "finalityDelayBlocks": blocknumber - lastFinalityBlockNumber };
                lastBlockTime = blocktime;
                blockList[index++] = blockInfo;
            }
        }else{
            var { isSigned, meta, method: { args, method, section } } = extrinsics[0];
            var blocktime = args[0];
    
            var { isSigned2, meta2, method: { args, method, section } } = extrinsics[1];
            var finalityBlockNumber = lastFinalityBlockNumber;
            if(method=="finalHint"){
                finalityBlockNumber = args[0];
                lastFinalityBlockNumber = finalityBlockNumber;
            }
                
            console.log(`blocknumber = ${blocknumber}, blocktime = ${blocktime}, extrinsic number = ${extrinsic_number}, finalityBlockNumber = ${finalityBlockNumber}`);
            if (lastBlockTime == 0) {
                lastBlockTime = blocktime;
                var blockInfo = { "blockNumber": blocknumber, "blockDeltTime": 0, "extrinsicNumber": extrinsic_number, "TPS": 0 , "finalityDelayBlocks": blocknumber - finalityBlockNumber};
                blockList[index++] = blockInfo;
            } else {
                var blockInfo = { "blockNumber": blocknumber, "blockDeltTime": blocktime - lastBlockTime, "extrinsicNumber": extrinsic_number, 
                "TPS": ((extrinsic_number * 1000) / (blocktime - lastBlockTime)).toFixed(2), "finalityDelayBlocks": blocknumber - finalityBlockNumber };
                lastBlockTime = blocktime;
                blockList[index++] = blockInfo;
            }
        }
    }

    const csv = new ObjectsToCsv(blockList);
    await csv.toDisk("TPS.csv");
}

getBlockInfo().catch(console.error).finally(() => process.exit());