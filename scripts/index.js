const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");

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

async function test() {
    await cryptoWaitReady(); // wait for WASM interface initialized
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const bob = keyring.addFromUri("//Bob");

    let nonce = new BN("1", 10);
    let session_id = new BN("42", 10);
    let base = new BN("1000000000000000", 10); // base = 1e15
    let amount = new BN("99", 10);
    let amt = amount.mul(base);
    let res = construct_byte_array(bob.publicKey, nonce, session_id, amt);
    let msg = blake2AsU8a(res);

    let signature = alice.sign(msg);
    let hexsig = toHexString(signature);
    console.log(`nonce: ${nonce}, session_id: ${session_id}, amt: ${amount}, signature: ${hexsig}`);
}

//test();
async function test1() {
    const wsProvider = new WsProvider("ws://127.0.0.1:9944");
    const api = await ApiPromise.create({
        provider: wsProvider,
    });

    let bal = await api.query.balances.totalIssuance();
    let acc1 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let bal1 = await api.query.system.account(acc1);
    let free1 = bal1.data.free.toString(10);
    let acc2 = "5GNkauE8C4HXo4UnvMCoCrzQKNhixD6oa4eDGYgiJKsDfFWM";
    acc2 = "5C4xaPznTFhENxuEqbuRMLh7aKuV3Jb8neRFLtV6dRM6xPs1"; // chao_stash_test
    let bal2 = await api.query.system.account(acc2);
    let free2 = bal2.data.free.toString(10);
    console.log(`total issuance is: ${bal}, account at ${acc1} has balance ${free1}, at ${acc2} has balance ${free2}`);
}

test1();
