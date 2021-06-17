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


// pallet credit test
async function initializeCreditScore(api, signer, score, test, expect) {
    const unsub = await api.tx.credit
        .initializeCreditExtrinsic(score)
        .signAndSend(signer, ({ events = [], status }) => {
            if (status.isFinalized) {
                events.forEach(({ phase, event: { data, method, section } }) => {
                    if (method == "ExtrinsicFailed") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": initializeCreditExtrinsic Failed, " + "expect " + expect);
                    } else if (method == "ExtrinsicSuccess") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": initializeCreditExtrinsic Success, " + "expect " + expect);
                    }
                });

                unsub();
            }
        });
}

async function killCreditScore(api, signer, test, expect) {
    const unsub = await api.tx.credit
        .killCreditExtrinsic()
        .signAndSend(signer, ({ events = [], status }) => {
            if (status.isFinalized) {
                events.forEach(({ phase, event: { data, method, section } }) => {
                    if (method == "ExtrinsicFailed") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": killCreditExtrinsic Failed, " + "expect " + expect);
                    } else if (method == "ExtrinsicSuccess") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": killCreditExtrinsic Success, " + "expect " + expect);
                    }
                });
            }
        });
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

// pallet delegating test
async function claimPayment(api, sender, receiver) {
    // open channel (sender -> receiver)
    await api.tx.micropayment.openChannel(receiver.address, new BN("1000000000000000000", 10),3600).signAndSend(sender, { nonce: -1 });
    var nonceNum = await api.query.micropayment.nonce([sender.address, receiver.address]);
    // claim payment 
    let base = new BN("1000000000000000", 10); // base = 1e15
    let sessionId = new BN((0).toString(), 10);
    let nonce_micropayment = new BN((nonceNum).toString(), 10);
    let amtNum = 50;
    let amount = new BN(amtNum.toString(), 10);
    let amt = amount.mul(base);
    let res = construct_byte_array(receiver.publicKey, nonce_micropayment, sessionId, amt);
    let msg = blake2AsU8a(res);
    let signature = sender.sign(msg);
    let hexsig = toHexString(signature);
    await api.tx.micropayment.claimPayment(sender.address, sessionId, amt, '0x' + hexsig).signAndSend(receiver, { nonce: -1 });
}

async function delegate_failed(api, signer, validators, test, expect) {
    //delegate to validators
    const unsub = await api.tx.delegating
        .delegate(validators)
        .signAndSend(signer, ({ events = [], status }) => {
            if (status.isFinalized) {
                events.forEach(({ phase, event: { data, method, section } }) => {
                    if (method == "ExtrinsicFailed") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": delegating Failed, " + "expect " + expect);
                    } else if (method == "ExtrinsicSuccess") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": delegating Success, " + "expect " + expect);
                    }
                });

                unsub();
            }
        });
}

async function delegate(api, signer, validators, test, expect) {
    const keyring = new Keyring({ type: "sr25519" });
    const dave = keyring.addFromUri("//Dave");
    const eve = keyring.addFromUri("//Eve");

    //accumulate credit score
    await claimPayment(api, dave, signer);
    await claimPayment(api, eve, signer);

    //wait for credit update
    console.log("wait 6m (An era)for credit score update...");
    await Utils.sleep(1000 * 360);

    let score = await api.query.credit.userCredit(signer.address);
    console.log(`score of account ${signer.address} is ${score.unwrapOr(0)}`);

    //delegate to validators
    const unsub = await api.tx.delegating
        .delegate(validators)
        .signAndSend(signer, ({ events = [], status }) => {
            if (status.isFinalized) {
                events.forEach(({ phase, event: { data, method, section } }) => {
                    if (method == "ExtrinsicFailed") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": delegating Failed, " + "expect " + expect);
                    } else if (method == "ExtrinsicSuccess") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": delegating Success, " + "expect " + expect);
                    }
                });

                unsub();
            }
        });
}

async function undelegate(api, signer, test, expect) {
    //delegate to validators
    const unsub = await api.tx.delegating
        .undelegate()
        .signAndSend(signer, ({ events = [], status }) => {
            if (status.isFinalized) {
                events.forEach(({ phase, event: { data, method, section } }) => {
                    if (method == "ExtrinsicFailed") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": undelegate Failed, " + "expect " + expect);
                    } else if (method == "ExtrinsicSuccess") {
                        if (test == 0)
                            console.log("Init success!");
                        else
                            console.log("Test #" + test + ": undelegate Success, " + "expect " + expect);
                    }
                });

                unsub();
            }
        });
}

async function poc_test() {
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

    
    // Failed: #1, pallet credit: initializeCreditExtrinsic() [30~ ] Failed
    console.log("Running test #1");
    await initializeCreditScore(api, alice, 30, 1, "Failed");
    await Utils.sleep(30 * 1000);

    // Success: #2, pallet credit: initializeCreditExtrinsic() [0~29] Success
    console.log("Running test #2");
    await initializeCreditScore(api, alice, 29, 2, "Success");
    await Utils.sleep(30 * 1000);

    // Failed: #3, pallet credit: recall initializeCreditExtrinsic() [0~29] Failed
    console.log("Running test #3");
    await initializeCreditScore(api, alice, 20, 3, "Failed");
    await Utils.sleep(30 * 1000);

    // Success: #4, pallet credit: killCreditExtrinsic() Success
    console.log("Running test #4");
    await killCreditScore(api, alice, 4, "Success");
    await Utils.sleep(30 * 1000);

    // Failed: #5, pallet credit: killCreditExtrinsic() Failed
    console.log("Running test #5");
    await killCreditScore(api, alice, 5, "Failed");
    await Utils.sleep(30 * 1000);
    
    // Failed: #6, pallet delegating: delegate() Failed
    // charlie score is too low
    console.log("Running test #6");
    await initializeCreditScore(api, charlie, 0, 6, "Success");
    await Utils.sleep(30 * 1000);
    await delegate_failed(api, charlie, [alice_stash.address, bob_stash.address], 6, "Failed");
    await Utils.sleep(30 * 1000);

    // Success: #7, pallet delegating: delegate() Success
    console.log("Running test #7");
    await delegate(api, charlie, [alice_stash.address, bob_stash.address], 7, "Success");
    await Utils.sleep(30 * 1000);

    // Success: #8, pallet delegating: undelegate() Success
    console.log("Running test #8");
    await undelegate(api, charlie, 8, "Success");
    await Utils.sleep(30 * 1000);


    // Failed: #9, pallet delegating: delegate() Failed
    // ferdie is not candidate validator
    console.log("Running test #9");
    await delegate_failed(api, charlie, [alice_stash.address, ferdie.address], 9, "Failed");
    await Utils.sleep(30 * 1000);
}

poc_test().catch(console.error).finally(() => process.exit());
