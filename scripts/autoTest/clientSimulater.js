const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");
const serverHost = "wss://10.168.98.1:443";

function toHexString(byteArray) {
    return Array.from(byteArray, function (byte) {
        return ("0" + (byte & 0xff).toString(16)).slice(-2);
    }).join("");
}

// nonce:u64, session_id:u32
function construct_byte_array(addr, nonce, session_id, amount) {
    let arr = [];
    nonce = nonce.toArray("be", 8);
    session_id = session_id.toArray("be", 4);
    amount = amount.toArray("le", 16); // amount is le encoded
    arr.push(...addr, ...nonce, ...session_id, ...amount);
    return arr;
}

function sleep(ms) {
    return new Promise(res => setTimeout(res, ms));
}
/*
send_rui: transaction sender's uri
type: transaction type, 
      0 balances transfer transaction
      1 micropayment transaction
*/
async function start_send_transaction(send_uri, type) {
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

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    console.log(`${send_uri}`);
    const sender = keyring.addFromUri(send_uri);
    const receiver = keyring.addFromUri(send_uri+"rec");

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);

    var record = {
        transaction_num: 0,
        start_time: Date.now(),
        end_time: 0
    };

    var nonceNum = 0;
    var sessionIdNum = 0;
    var amtNum = 1;

    if (type == 1) {
        await api.tx.micropayment.openChannel(receiver.address, 3600).signAndSend(sender, { nonce: -1 });
        nonceNum = await api.query.micropayment.nonce([sender.address, receiver.address]);
        console.log(`nonce = ${nonceNum}`);
        sessionIdNum = 0;
    }

    while (true) {
        var i = 0;
        var start_time = Date.now();
        var end_time = 0;

        try {
            for (; i < 10000; i++) {
                if (type == 1) {
                    let nonce_micropayment = new BN((nonceNum).toString(), 10);
                    let sessionId = new BN((sessionIdNum+i).toString(), 10);
                    let base = new BN("1000000000000", 10); // base = 1e12  0.001 Unit
                    let amount = new BN(amtNum.toString(), 10);
                    let amt = amount.mul(base);
                    let res = construct_byte_array(receiver.publicKey, nonce_micropayment, sessionId, amt);
                    let msg = blake2AsU8a(res);
                    let signature = sender.sign(msg);
                    let hexsig = toHexString(signature);
                    //console.log(`nonce: ${nonce_micropayment}, session_id: ${sessionId}, amt: ${amount}, signature: ${hexsig}`);
                    await api.tx.micropayment.claimPayment(sender.address, sessionId, amt, '0x' + hexsig).signAndSend(receiver, { nonce: -1 });
                } else {
                    const txhash = await api.tx.balances
                        .transfer(receiver.address, 100000000000)
                        .signAndSend(sender, { nonce: -1 });
                }
            }
        } catch (error) {
            //console.log(error);
            end_time = Date.now();
            console.log(`time interval ${end_time - start_time}ms, send transfer ${i}`);
            await sleep(2000);
            record.end_time = Date.now();
            record.transaction_num += i;
            console.log(`time interval ${((record.end_time - record.start_time) / 1000).toFixed(2)}s, send transfer ${record.transaction_num}, send TPS ${(record.transaction_num * 1000 / (record.end_time - record.start_time)).toFixed(2)}`);
        }finally{
            if (type == 1) {
               // await api.tx.micropayment.closeChannel(sender.address).signAndSend(receiver);
            }
        }
    }
}

start_send_transaction(process.argv[2], process.argv[3]).catch(console.error).finally(() => process.exit());