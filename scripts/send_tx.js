const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");

async function test() {
    const wsProvider = new WsProvider("ws://127.0.0.1:9944");
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            Balance: "u128",
            Timestamp: "Moment",
            BlockNumber: "u32",
            Node: {
                account_id: "AccountId",
                ipv4: "Vec<u8>",
                country: "Vec<u8>",
                expire: "BlockNumber",
            },
            ChannelOf: {
                sender: "AccountId",
                receiver: "AccountId",
                balance: "Balance",
                nonce: "u64",
                opened: "BlockNumber",
                expiration: "BlockNumber",
            },
            CreditDelegateInfo: {
                delegator: "AccountId",
                score: "u64",
                validators: "Vec<AccountId>",
            },
        },
    });
    const keyring = new Keyring({ type: "sr25519" });

    const sender = keyring.addFromUri("//Alice");
    const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    // Get the nonce for the admin key
    const { nonce } = await api.query.system.account(ALICE);
    //const recipient = "5GNkauE8C4HXo4UnvMCoCrzQKNhixD6oa4eDGYgiJKsDfFWM";
    const recipient = "5C4xaPznTFhENxuEqbuRMLh7aKuV3Jb8neRFLtV6dRM6xPs1"; // chao_stash_test
    const AMT = 111;

    console.log(`sending ${AMT} from  ${ALICE} to ${recipient} with nonce ${nonce}`);

    api.tx.balances.transfer(recipient, AMT).signAndSend(sender, { nonce }, ({ events = [], status }) => {
        console.log("Transaction status:", status.type);

        if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");

            events.forEach(({ event: { data, method, section }, phase }) => {
                console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
            });
        } else if (status.isFinalized) {
            console.log("Finalized block hash", status.asFinalized.toHex());

            process.exit(0);
        }
    });
}

test().catch(console.err);
