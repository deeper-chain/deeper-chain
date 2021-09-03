const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const to = require("await-to-js").default;
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");
const Utils = require("./utils.js");

//const serverHost = "wss://138.68.229.14:443";
//const serverHost = "wss://10.168.98.1:443";
const serverHost = "ws://127.0.0.1:9944";
const DPR = new BN("1000000000000000", 10); // base = 1e15 according to frontend apps, in code it's 1e14, fix it later;
const ONE_MILLION = 1000000;
// convert number in "atom" unit to DPR unit
// return float
function atomToDPR(amt) {
    return amt / DPR;
  }
  
// convert number in DPR unit to smallest "atom" unit
function DPRToAtom(amt) {
    return BigInt(parseInt(amt * ONE_MILLION)) * BigInt(DPR / ONE_MILLION);
}

function toHexString(byteArray) {
    return Array.from(byteArray, function (byte) {
        return ("0" + (byte & 0xff).toString(16)).slice(-2);
    }).join("");
}

// nonce:u64, session_id:u32
function construct_byte_array(addr, nonce, senderPubkey) {
    let arr = [];
    nonce = new BN(nonce.toString(), 10);
    nonce = nonce.toArray("be", 8);
    arr.push(...addr, ...nonce, ...senderPubkey);
    return arr;
}

function claimPayment(api, sender, receiver, nonceNum, sessionIdNum, amtNum) {
    return new Promise(function (resolve, reject) {
        let nonce = new BN(nonceNum.toString(), 10);
        let sessionId = new BN(sessionIdNum.toString(), 10);
        let amount = new BN(amtNum.toString(), 10);
        let amt = amount.mul(DPR);
        let res = construct_byte_array(receiver.publicKey, nonce, sessionId, amt);
        let msg = blake2AsU8a(res);
        let signature = sender.sign(msg);
        let hexsig = toHexString(signature);
        console.log(`nonce: ${nonce}, session_id: ${sessionId}, amt: ${amount}, signature: ${hexsig}`);
        api.tx.micropayment.claimPayment(sender.address, sessionId, amt, "0x" + hexsig).signAndSend(receiver, {nonce: -1}, ({ events = [], status }) => {
            //console.log("Transaction status:", status.type);
            if (status.isInBlock) {
                //console.log("Included at block hash", status.asInBlock.toHex());
                //console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    //console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                //console.log("Finalized block hash", status.asFinalized.toHex());
                resolve();
            }
        });
    });
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
    //set atomos pubkey
    await api.tx.sudo.sudo(
      api.tx.micropayment.setAtmosPubkey(atomsKeypair.address)
    ).signAndSend(root, { nonce: -1 });

    //set charliecreditData
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
    //charlie iamOnlie
    await api.tx.deeperNode.imOnline().signAndSend(alice, { nonce: -1 });

    //await Utils.sleep(1000* 60 * 19);
    
    // charlie send message and signature which is signed by atomsKeypair
    let alice_nonce_option = await api.query.micropayment.atmosNonce(alice.address); 
    let alice_nonce = alice_nonce_option.unwrapOr(0);

    let res = construct_byte_array(atomsKeypair.publicKey, alice_nonce, alice.publicKey);
    let msg = blake2AsU8a(res);
    let signature = atomsKeypair.sign(msg);
    let hexsig = "0x" + toHexString(signature);
    console.log(`nonce: ${alice_nonce}, signature: ${hexsig}`);
    await api.tx.micropayment.addCreditByTraffic(alice_nonce, hexsig)
        .signAndSend(alice, { nonce: -1 });
    
    await Utils.sleep(1000* 60 * 3);
}

addCreditByTrafficTest().catch(console.error).finally(() => process.exit());