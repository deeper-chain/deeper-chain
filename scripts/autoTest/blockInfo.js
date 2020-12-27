const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");

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

    // returns Hash
    const blockHash = await api.rpc.chain.getBlockHash(73);
    // returns SignedBlock
    const signedBlock = await api.rpc.chain.getBlock(blockHash);

    // the hash for the block, always via header (Hash -> toHex()) - will be
    // the same as blockHash above (also available on any header retrieved,
    // subscription or once-off)
    console.log(signedBlock.block.header.hash.toHex());

    // the hash for each extrinsic in the block
    signedBlock.block.extrinsics.forEach((ex, index) => {
        // the extrinsics are decoded by the API, human-like view
        //console.log(index, ex.toHuman());

        const { isSigned, meta, method: { args, method, section } } = ex;

        // explicit display of name, args & documentation
        console.log(`${section}.${method}(${args.map((a) => a.toString()).join(', ')})`);
        console.log(meta.documentation.map((d) => d.toString()).join('\n'));

        // signer/nonce info
        if (isSigned) {
            console.log(`signer=${ex.signer.toString()}, nonce=${ex.nonce.toString()}`);
        }
    });

    const allRecords = await api.query.system.events.at(signedBlock.block.header.hash);
    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
        // filter the specific events based on the phase and then the
        // index of our extrinsic in the block
        const events = allRecords
            .filter(({ phase }) =>
                phase.isApplyExtrinsic &&
                phase.asApplyExtrinsic.eq(index)
            )
            .map(({ event }) => `${event.section}.${event.method}`);

        console.log(`${section}.${method}:: ${events.join(', ') || 'no events'}`);
    });

    // Loop through the Vec<EventRecord>
    allRecords.forEach((record) => {
        // Extract the phase, event and the event types
        const { event, phase } = record;
        const types = event.typeDef;

        if (event.section == "staking" && event.method == "EraPayout") {
            // Show what we are busy with
            console.log(`\t${event.section}:${event.method}:: (phase=${phase.toString()})`);
            console.log(`\t\t${event.meta.documentation.toString()}`);

            // Loop through each of the parameters, displaying the type and data
            event.data.forEach((data, index) => {
                console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
            });
        }
    });
}

getBlockInfo().catch(console.error).finally(() => process.exit());