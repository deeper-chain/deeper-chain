const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { blake2AsU8a } = require('@polkadot/util-crypto');
const to = require('await-to-js').default;
const stringToU8a = require('@polkadot/util/string/toU8a').default;
const BN = require('bn.js');
const Utils = require('./utils.js');

const serverHost = 'ws://127.0.0.1:9944';
const DPR = new BN('1000000000000000000'); // base = 1e18;

// convert number in the smallest unit to DPR unit
function toDPR(amt) {
    return new BN(amt).div(DPR);
}

// convert number in DPR unit to number in the smallest unit of the currency.
function fromDPR(amt) {
    return new BN(amt).mul(DPR);
}

function toHexString(byteArray) {
    return '0x' + Array.from(byteArray, function (byte) {
        return ('0' + (byte & 0xff).toString(16)).slice(-2);
    }).join('');
}

// nonce:u64, session_id:u32
function construct_byte_array(addr, nonce, session_id, amount) {
    let arr = [];
    nonce = nonce.toArray('be', 8);
    session_id = session_id.toArray('be', 4);
    amount = amount.toArray('le', 16); // amount is le encoded
    arr.push(...addr, ...nonce, ...session_id, ...amount);
    return arr;
}

async function openChannel(api, sender, receiver, amt, duration) {
    const unsub = await api.tx.micropayment.openChannel(receiver.address, amt, duration)
        .signAndSend(sender, { nonce: -1 }, ({ status }) => {
            if (status.isFinalized) {
                unsub();
            }
        });
}

async function closeChannel(api, sender, receiver) {
    const unsub = await api.tx.micropayment.closeChannel(sender.address)
        .signAndSend(receiver, { nonce: -1 }, ({ status }) => {
            if (status.isFinalized) {
                unsub();
            }
        });
}

async function claimPayment(api, sender, receiver, nonceNum, sessionIdNum, amount) {
    let nonce = new BN(nonceNum);
    let sessionId = new BN(sessionIdNum);
    let amt = new BN(amount).mul(DPR);
    let res = construct_byte_array(receiver.publicKey, nonce, sessionId, amt);
    let msg = blake2AsU8a(res);
    let sig = sender.sign(msg);
    console.log(`nonce: ${nonce}, session_id: ${sessionId}, amount: ${amount}`);
    const unsub = await api.tx.micropayment.claimPayment(sender.address, sessionId, amt, toHexString(sig))
        .signAndSend(receiver, { nonce: -1 }, ({ status }) => {
            if (status.isFinalized) {
                unsub();
            }
        });
}

async function micropayment_test() {
    //connect to chain
    const api = await Utils.get_api(serverHost);
    // accounts
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const alice_stash = keyring.addFromUri('//Alice//stash');

    const bob = keyring.addFromUri('//Bob');
    const bob_stash = keyring.addFromUri('//Bob//stash');

    const charlie = keyring.addFromUri('//Charlie');
    const dave = keyring.addFromUri('//Dave');
    const eve = keyring.addFromUri('//Eve');
    const ferdie = keyring.addFromUri('//Ferdie');


    // open channel Alice -> Bob
    console.log('OpenChannel [Alice -> Bob]');
    await openChannel(api, alice, bob, fromDPR(100), 1000);
    await Utils.sleep(30 * 1000);
    let totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    let totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    let result = totalMicropaymentChannelBalance == 100 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // open channel Alice->Charlie
    console.log('OpenChannel [Alice -> Charlie]');
    await openChannel(api, alice, charlie, fromDPR(100), 1000);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 200 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // open channel Alice -> Dave
    console.log('OpenChannel [Alice -> Dave]');
    await openChannel(api, alice, dave, fromDPR(100), 1000);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 300 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Bob claim from Alice
    console.log('ClaimPayment [Alice -> Bob]');
    let nonce = await api.query.micropayment.nonce([alice.address, bob.address]);
    let sidOption = await api.query.micropayment.sessionId([alice.address, bob.address]);
    let sid = sidOption.unwrapOr(0);
    await claimPayment(api, alice, bob, nonce, sid + 1, 10);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 290 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Bob close channel [Alice -> Charlie]
    console.log('close channel [Alice -> Charlie]');
    await closeChannel(api, alice, charlie);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 190 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    // Dave add balance to channel [Alice -> Dave]
    console.log('addBalance to channel [Alice -> Dave]');
    await api.tx.micropayment.addBalance(dave.address, fromDPR(30)).signAndSend(alice, { nonce: -1 });
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 220 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);

    await closeChannel(api, alice, bob);
    await closeChannel(api, alice, dave);
    await Utils.sleep(30 * 1000);
    totalMicropaymentChannelBalanceOption = await api.query.micropayment.totalMicropaymentChannelBalance(alice.address);
    totalMicropaymentChannelBalance = toDPR(totalMicropaymentChannelBalanceOption.unwrapOr(0));
    result = totalMicropaymentChannelBalance == 0 ? 'OK' : 'Failed';
    console.log(`total channel balance = ${totalMicropaymentChannelBalance}, test ${result}`);
}

micropayment_test().catch(console.error).finally(() => process.exit());
