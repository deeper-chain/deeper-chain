const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { blake2AsU8a, secp256k1KeypairFromSeed, cryptoWaitReady } = require("@polkadot/util-crypto");
const to = require("await-to-js").default;
const stringToU8a = require("@polkadot/util/string/toU8a").default;
const BN = require("bn.js");

//const serverHost = "wss://138.68.229.14:443";
//const serverHost = "wss://10.168.98.1:443";
const serverHost = "ws://127.0.0.1:9944";
const DPR = new BN("1000000000000000", 10); // base = 1e15 according to frontend apps, in code it's 1e14, fix it later;

const delay_promise = function (ms) {
    return new Promise(function (resolve, reject) {
        setTimeout(() => {
            reject(`Timeout in ${ms} ms`);
        }, ms);
    });
};

async function get_api(url_string) {
    // example of url_string: wss://138.68.229.14:443
    process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
    const wsProvider = new WsProvider(url_string);
    console.log("start wsprovider...");

    let promiseA = ApiPromise.create({
        provider: wsProvider,
        types: {
            Balance: "u128",
            Timestamp: "Moment",
            BlockNumber: "u32",
            CountryRegion: "Vec<u8>",
            IpV4: "Vec<u8>",
            Node: {
                account_id: "AccountId",
                ipv4: "IpV4",
                country: "CountryRegion",
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

    let promiseB = delay_promise(30000);

    let race = Promise.race([promiseA, promiseB]);
    let [err, api] = await to(race);
    console.log(`hehe err: ${err}`);
    console.log(`hehe api: ${api}`);
    return api;
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
        api.tx.micropayment.openChannel(receiver.address, amt, duration).signAndSend(sender, ({ events = [], status }) => {
            console.log("Transaction status:", status.type);
            if (status.isInBlock) {
                console.log("Included at block hash", status.asInBlock.toHex());
                console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                console.log("Finalized block hash", status.asFinalized.toHex());
                resolve();
            }
        });
    });
}
function closeChannel(api, sender, receiver) {
    return new Promise(function (resolve, reject) {
        api.tx.micropayment.closeChannel(sender.address).signAndSend(receiver, ({ events = [], status }) => {
            console.log("Transaction status:", status.type);
            if (status.isInBlock) {
                console.log("Included at block hash", status.asInBlock.toHex());
                console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                console.log("Finalized block hash", status.asFinalized.toHex());
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
        api.tx.micropayment.claimPayment(sender.address, sessionId, amt, "0x" + hexsig).signAndSend(receiver, ({ events = [], status }) => {
            console.log("Transaction status:", status.type);
            if (status.isInBlock) {
                console.log("Included at block hash", status.asInBlock.toHex());
                console.log("Events:");
                events.forEach(({ event: { data, method, section }, phase }) => {
                    console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                console.log("Finalized block hash", status.asFinalized.toHex());
                resolve();
            }
        });
    });
}

async function claimPayment_test(amtStr) {
    const api = await get_api(serverHost);
    await cryptoWaitReady();
    // accounts
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const bob = keyring.addFromUri("//Bob");
    const charlie = keyring.addFromUri("//Charlie");
    const dave = keyring.addFromUri("//Dave");

    console.log(`before open channel`);
    await printFreeBalance(api, alice.address);
    let amount = new BN(amtStr, 10);
    let amt = amount.mul(DPR);
    await openChannel(api, alice, bob, amt, 7); // 7 days

    console.log(`after open channel`);
    await printFreeBalance(api, alice.address);
    await printFreeBalance(api, bob.address);
    let nonce = 0;
    let sessionId = 1;
    let delta = 20;
    let times = 7;
    let i = 0;
    for (i = 0; i < times; i++) {
        await claimPayment(api, alice, bob, nonce, sessionId + i, delta);
    }
    //await claimPayment(api, charlie, bob, nonce, sessionId, delta);
    //await claimPayment(api, alice, bob, nonce, sessionId + 2, delta);
    console.log(`after claim payments ${delta * times}`);
    await printFreeBalance(api, alice.address);
    await printFreeBalance(api, bob.address);

    await closeChannel(api, alice, bob);
    console.log(`after close channel`);
    await printFreeBalance(api, alice.address);
    await printFreeBalance(api, bob.address);
}

async function printFreeBalance(api, address) {
    let bal = await api.query.system.account(address);
    let amt = new BN(bal.data.free.toString(10), 10);
    let free = amt / DPR;
    console.log(`free balance of ${address} is ${free}`);
}

//-------------------------------------------------------------------------------------

async function functionalTest_credit() {
    // connect to chain
    const api = await get_api(serverHost);
    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    // init Credit
    api.tx.credit.updateCreditExtrinsic(90).signAndSend(charlie, ({ events = [], status }) => {
        console.log("Transaction status:", status.type);
        if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
                console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
            });
        } else if (status.isFinalized) {
            console.log("Finalized block hash", status.asFinalized.toHex());
        }
    });
    //let score = await api.query.credit.userCredit(charlie.address);
    //console.log(`Charlie updateCreditExtrinsic OK ${score.unwrap()}`);
    api.tx.credit.updateCreditExtrinsic(88).signAndSend(alice);
    api.tx.credit.updateCreditExtrinsic(87).signAndSend(bob);
    api.tx.credit.updateCreditExtrinsic(89).signAndSend(dave);
    api.tx.credit.updateCreditExtrinsic(90).signAndSend(eve);
    api.tx.credit.updateCreditExtrinsic(80).signAndSend(ferdie);
}

async function functionalTest_credit_check() {
    // connect to chain
    const api = await get_api(serverHost);

    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    // check credit score

    let score = await api.query.credit.userCredit(charlie.address);
    if (score.unwrap() == 90) console.log(`Charlie updateCreditExtrinsic OK ${score.unwrap()}`);

    score = await api.query.credit.userCredit(alice.address);
    if (score.unwrap() == 88) console.log(`Alice updateCreditExtrinsic OK ${score.unwrap()}`);

    score = await api.query.credit.userCredit(bob.address);
    if (score.unwrap() == 87) console.log(`Bob updateCreditExtrinsic OK ${score.unwrap()}`);

    score = await api.query.credit.userCredit(dave.address);
    if (score.unwrap() == 89) console.log(`Dave updateCreditExtrinsic OK ${score.unwrap()}`);

    score = await api.query.credit.userCredit(eve.address);
    if (score.unwrap() == 89) console.log(`Eve updateCreditExtrinsic OK ${score.unwrap()}`);

    score = await api.query.credit.userCredit(ferdie.address);
    if (score.unwrap() == 89) console.log(`Ferdie updateCreditExtrinsic OK ${score.unwrap()}`);
}

async function functionalTest_delegate() {
    // connect to chain
    const api = await get_api(serverHost);

    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    //console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    //console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    //console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    //console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    //console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    //console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    //console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    //console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    //console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    // Delegating
    api.tx.delegating.delegate(alice_stash.address).signAndSend(charlie, ({ events = [], status }) => {
        console.log("Transaction status:", status.type);
        if (status.isInBlock) {
            console.log("Included at block hash", status.asInBlock.toHex());
            console.log("Events:");
            events.forEach(({ event: { data, method, section }, phase }) => {
                console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
            });
        } else if (status.isFinalized) {
            console.log("Finalized block hash", status.asFinalized.toHex());
        }
    });

    api.tx.delegating.delegate(alice_stash.address).signAndSend(alice);
    api.tx.delegating.delegate(bob_stash.address).signAndSend(bob);
    api.tx.delegating.delegate(chao0_stash.address).signAndSend(dave);
    api.tx.delegating.delegate(chao1_stash.address).signAndSend(eve);
    api.tx.delegating.delegate(chao2_stash.address).signAndSend(ferdie);
}

async function functionalTest_delegate_check() {
    // connect to chain
    const api = await get_api(serverHost);

    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    //console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    //console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    //console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    //console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    //console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    //console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    //console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    //console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    //console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    // Delegating

    let ledger = await api.query.delegating.creditLedger(alice.address);
    //console.log(`Alice creditLedger: ${ledger}`);
    if (ledger.validatorAccount == alice_stash.address) console.log(`Alice has delegated score to Alice_stash OK`);

    ledger = await api.query.delegating.creditLedger(bob.address);
    //console.log(`Bob creditLedger: ${ledger}`);
    if (ledger.validatorAccount == bob_stash.address) console.log(`Bob has delegated score to bob_stash OK`);

    ledger = await api.query.delegating.creditLedger(charlie.address);
    //console.log(`Charlie creditLedger: ${ledger}`);
    if (ledger.validatorAccount == alice_stash.address) console.log(`Charlie has delegated score to Alice_stash OK`);

    ledger = await api.query.delegating.creditLedger(dave.address);
    //console.log(`Dave creditLedger: ${ledger}`);
    if (ledger.validatorAccount == chao0_stash.address) console.log(`Dave has delegated score to Chao0_stash OK`);

    ledger = await api.query.delegating.creditLedger(eve.address);
    //console.log(`Eve creditLedger: ${ledger}`);
    if (ledger.validatorAccount == chao1_stash.address) console.log(`Eve has delegated score to Chao1_stash OK`);

    ledger = await api.query.delegating.creditLedger(ferdie.address);
    //console.log(`Ferdie creditLedger: ${ledger}`);
    if (ledger.validatorAccount == chao2_stash.address) console.log(`Ferdie has delegated score to Chao2_stash OK`);

    let currentEra = await api.query.delegating.currentEra();
    let era = currentEra.unwrap();
    console.log(`current era is ${era}`);

    let delegators = await api.query.delegating.delegators(era, alice_stash.address);
    console.log(`Alice_Stash delegators: ${delegators} in Era ${era}`);

    delegators = await api.query.delegating.delegators(era, bob_stash.address);
    console.log(`Bob_Stash delegators: ${delegators} in Era ${era}`);

    delegators = await api.query.delegating.delegators(era, chao0_stash.address);
    console.log(`Chao0_Stash delegators: ${delegators} in Era ${era}`);

    delegators = await api.query.delegating.delegators(era, chao1_stash.address);
    console.log(`Chao1_Stash delegators: ${delegators} in Era ${era}`);

    delegators = await api.query.delegating.delegators(era, chao2_stash.address);
    console.log(`Chao2_Stash delegators: ${delegators} in Era ${era}`);
}

async function functionalTest_credit_attenuate_set() {
    // connect to chain
    const api = await get_api(serverHost);
    // accounts
    const keyring = new Keyring({ type: "sr25519" });

    const alice = keyring.addFromUri("//Alice");
    const alice_stash = keyring.addFromUri("//Alice//stash");
    //console.log(`Alice: ${alice.address}, Alice_Stash: ${alice_stash.address}`);

    const bob = keyring.addFromUri("//Bob");
    const bob_stash = keyring.addFromUri("//Bob//stash");
    //console.log(`Bob: ${bob.address}, Bob_Stash: ${bob_stash.address}`);

    const charlie = keyring.addFromUri("//Charlie");
    //console.log(`Charlie: ${charlie.address}`);

    const dave = keyring.addFromUri("//Dave");
    //console.log(`Dave: ${dave.address}`);

    const eve = keyring.addFromUri("//Eve");
    //console.log(`Eve: ${eve.address}`);

    const ferdie = keyring.addFromUri("//Ferdie");
    //console.log(`Ferdie: ${ferdie.address}`);

    const chao0 = keyring.addFromMnemonic("wet wait more hammer glass drastic reform detect corn resource lake bomb");
    const chao0_stash = keyring.addFromUri("wet wait more hammer glass drastic reform detect corn resource lake bomb//stash");
    //console.log(`Chao0: ${chao0.address}, Chao0_stash: ${chao0_stash.address}`);

    const chao1 = keyring.addFromMnemonic("license trigger sight gallery trophy before rough village clean become blur blast");
    const chao1_stash = keyring.addFromUri("license trigger sight gallery trophy before rough village clean become blur blast//stash");
    //console.log(`Chao1: ${chao1.address}, Chao1_stash: ${chao1_stash.address}`);

    const chao2 = keyring.addFromMnemonic("discover despair state general virtual method ten someone rookie learn damage artefact");
    const chao2_stash = keyring.addFromUri("discover despair state general virtual method ten someone rookie learn damage artefact//stash");
    //console.log(`Chao2: ${chao2.address}, Chao2_stash: ${chao2_stash.address}`);

    // registerDevice
    api.tx.deeperNode.registerDevice("0x1234", 1).signAndSend(charlie);
    api.tx.deeperNode.registerDevice("0x1234", 2).signAndSend(charlie);
    api.tx.deeperNode.registerDevice("0x1234", 3).signAndSend(charlie);
}

// to run this js
// NODE_TLS_REJECT_UNAUTHORIZED=0 node index.js

var args = process.argv.slice(2);
// micropayment test
claimPayment_test(args[0])
    .catch(console.error)
    .finally(() => process.exit());

// credit pallet test
//functionalTest_credit();
//setTimeout(functionalTest_credit_check, 30000);
