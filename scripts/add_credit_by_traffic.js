const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const to = require("await-to-js").default;
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");
const Utils = require("./utils.js");

const serverHost = "ws://127.0.0.1:9944";

function toHexString(byteArray) {
    return Array.from(byteArray, function (byte) {
        return ("0" + (byte & 0xff).toString(16)).slice(-2);
    }).join("");
}

function construct_byte_array(addr, nonce, senderPubkey) {
    let arr = [];
    nonce = new BN(nonce.toString(), 10);
    nonce = nonce.toArray("be", 8);
    arr.push(...addr, ...nonce, ...senderPubkey);
    return arr;
}

async function addCreditByTrafficTest() {
    //connect to chain
    const api = await Utils.get_api(serverHost);
    // accounts
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const bob = keyring.addFromUri("//Bob");
    const charlie = keyring.addFromUri("//Charlie");

    // sudo set Atomos Accountid
    let root = alice;
    let atomsKeypair = keyring.addFromUri("rookie pet ramp sniff vibrant silent liquid burden push shield police ripple");
    //set balance
    await api.tx.sudo.sudo(
      api.tx.balances.setBalance(atomsKeypair.address, new BN("2000000000000000000", 10), 0) // 2DPR
    ).signAndSend(root, { nonce: -1 });
    atomsKeypair = bob;
    //set atomos pubkey
    await api.tx.sudo.sudo(
      api.tx.creditAccumulation.setAtmosPubkey(atomsKeypair.address)
    ).signAndSend(root, { nonce: -1 });

    //set Alice creditData
    const creditData = {
        campaign_id: 0,
        credit: 300,
        initial_credit_level: 3,
        rank_in_initial_credit_level: 1,
        number_of_referees: 0,
        current_credit_level: 3,
        reward_eras: 270,
      };
    await api.tx.sudo.sudo(
      api.tx.credit.addOrUpdateCreditData(alice.address, creditData)
    ).signAndSend(root, { nonce: -1 });
    //Alice iamOnlie
    await api.tx.deeperNode.imOnline().signAndSend(alice, { nonce: -1 });

    //await Utils.sleep(1000* 60 * 19);
    
    // alice send message and signature which is signed by atomsKeypair
    let alice_nonce_option = await api.query.creditAccumulation.atmosNonce(alice.address); 
    let alice_nonce = alice_nonce_option.unwrapOr(0);

    let res = construct_byte_array(atomsKeypair.publicKey, alice_nonce, alice.publicKey);
    let msg = blake2AsU8a(res);
    let signature = atomsKeypair.sign(msg);
    let hexsig = "0x" + toHexString(signature);
    console.log(`nonce: ${alice_nonce}, signature: ${hexsig}`);
    await api.tx.creditAccumulation.addCreditByTraffic(alice_nonce, hexsig)
        .signAndSend(alice, { nonce: -1 });
    
    await Utils.sleep(1000* 60 * 3);
}

addCreditByTrafficTest().catch(console.error).finally(() => process.exit());