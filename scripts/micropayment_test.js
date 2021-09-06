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
function construct_byte_array(addr, nonce, session_id, amount) {
    let arr = [];
    nonce = nonce.toArray("be", 8);
    session_id = session_id.toArray("be", 4);
    amount = amount.toArray("le", 16); // amount is le encoded
    arr.push(...addr, ...nonce, ...session_id, ...amount);
    return arr;
}

function openChannel(api, sender, receiver, amt, duration) {
    return new Promise(function (resolve, reject) {
        api.tx.micropayment.openChannel(receiver.address, amt, duration).signAndSend(sender, {nonce: -1}, ({ events = [], status }) => {
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

function closeChannel(api, sender, receiver) {
    return new Promise(function (resolve, reject) {
        api.tx.micropayment.closeChannel(sender.address).signAndSend(receiver, {nonce: -1}, ({ events = [], status }) => {
            //console.log("Transaction status:", status.type);
            if (status.isInBlock) {
                //console.log("Included at block hash", status.asInBlock.toHex());
                //console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    //onsole.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                //console.log("Finalized block hash", status.asFinalized.toHex());
                resolve();
            }
        });
    });
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

async function micropayment_test() {
    //connect to chain
    const api = await Utils.get_api(serverHost);
    // accounts
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");

    const charlie = keyring.addFromUri("//Charlie");
    const dave = keyring.addFromUri("//Dave");
    const eve = keyring.addFromUri("//Eve");
    const ferdie = keyring.addFromUri("//Ferdie");

    
    // open channel Alice -> Bob
    console.log("OpenChannel [Alice -> Bob]");
    await openChannel(api, alice, bob, DPRToAtom(100), 1);
    await Utils.sleep(30 * 1000);
    let totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    let totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    let result = totalMicropaymentChannelBalance==100 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);
    
    // open channel Alice->Charlie
    console.log("OpenChannel [Alice -> Charlie]");
    await openChannel(api, alice, charlie, DPRToAtom(100), 1);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==200 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // open channel Alice -> Dave
    console.log("OpenChannel [Alice -> Dave]");
    await openChannel(api, alice, dave, DPRToAtom(100), 1);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==300 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Bob claim from Alice
    console.log("ClaimPayment [Alice-> Bob]");
    let nonce = await api.query.micropayment.nonce([alice.address, bob.address]);
    let sidOption = await api.query.micropayment.sessionId([alice.address, bob.address]);
    let sid = sidOption.unwrapOr(0);
    await claimPayment(api, alice, bob, nonce, sid + 1, 10);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==290 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Bob close channel [Alice -> Charlie]
    console.log("close channel [Alice -> Charlie]");
    await closeChannel(api, alice, charlie);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==190 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Dave addbalance to channel [Alice -> Dave]
    console.log("addBalance to channel [Alice -> Dave]");
    await api.tx.micropayment.addBalance(dave.address, DPRToAtom(30)).signAndSend(alice, {nonce: -1});
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==220 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    await closeChannel(api, alice, bob);
    await closeChannel(api, alice, dave);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = atomToDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance==0 ? "OK":"Failed";
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);
}

micropayment_test().catch(console.error).finally(() => process.exit());
