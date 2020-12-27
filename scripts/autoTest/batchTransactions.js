const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const ObjectsToCsv = require('objects-to-csv');
const child_process = require('child_process');
const BN = require("bn.js");
const serverHost = "wss://10.168.98.1:443";



async function batchTransfer(sendPrefix, num, type) {
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

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    // Get the current sudo key in the system
    const sudoKey = await api.query.sudo.key();
    console.log(`${sudoKey}`);
    const sudoPair = keyring.getPair(sudoKey.toString());

    for (var i = 0; i < num; i++) {
        const sender = keyring.addFromUri(sendPrefix + i.toString());
        await api.tx.sudo
            .sudo(
                api.tx.balances.setBalance(sender.address, new BN("10000000000000000000", 10), 0)
            )
            .signAndSend(sudoPair, { nonce: -1 });

        const receiver = keyring.addFromUri(sendPrefix + i.toString() + "rec");
        await api.tx.sudo
        .sudo(
            api.tx.balances.setBalance(receiver.address, new BN("10000000000000000000", 10), 0)
        )
        .signAndSend(sudoPair, { nonce: -1 });
    }
    console.log(`${num} accounts intilized!`);


    console.log("start to send transactions");
    for (var i = 0; i < num; i++) {
        var worker_process = child_process.fork("clientSimulater.js", [sendPrefix + i.toString(), type]);

        worker_process.on('close', function (code) {
            console.log('子进程已退出，退出码 ' + code);
        });
    }

}

// argv[2] : sender account URI prefix: "alice","ali"
// argv[3] : process numbers: 1,20,...50
// argv[4] : transaction type: 0 for transfer transaction, 1 for micropayment
batchTransfer(process.argv[2], process.argv[3], process.argv[4]);//.catch(console.error).finally(() => process.exit());