const {
  ApiPromise,
  WsProvider
} = require('@polkadot/api');
const { blake2AsU8a } = require('@polkadot/util-crypto');
const deeperChainTypes = require('./types.json');
const BN = require('bn.js')
const DPR = new BN('1000000000000000000'); // base = 1e18;

let $api;
async function getApi() {
  if ($api && $api.isConnected) {
    return $api
  }
  const wsProvider = new WsProvider('ws://127.0.0.1:9944')
  $api = await ApiPromise.create({
    provider: wsProvider,
    types: deeperChainTypes
  });
  return $api
}

async function openChannel(sender, receiver, amt, duration) {
  let api = await getApi();
  return new Promise(async (resolve, reject) => {
    const unsub = await api.tx.micropayment.openChannel(receiver.address, amt, duration)
      .signAndSend(sender, {
        nonce: -1
      }, ({
        status,
        events
      }) => {
        if (status.isInBlock || status.isFinalized) {
          events
            .forEach(({
              event: {
                data: [result]
              }
            }) => {
              // Now we look to see if the extrinsic was actually successful or not...
              if (result.isError) {
                let error = result.asError;
                if (error.isModule) {
                  // for module errors, we have the section indexed, lookup
                  const decoded = api.registry.findMetaError(error.asModule);
                  const {
                    docs,
                    name,
                    section
                  } = decoded;
                  console.log(`${section}.${name}: ${docs.join(' ')}`);
                  reject(`${section}.${name}: ${docs.join(' ')}`)
                } else {
                  // Other, CannotLookup, BadOrigin, no extra info
                  console.log(error.toString());
                  reject(`${section}.${name}: ${docs.join(' ')}`)
                }
              }
            });
          unsub();
          resolve(status.hash)
        }
      });
  })

}

function toHexString(byteArray) {
  return '0x' + Array.from(byteArray, function (byte) {
      return ('0' + (byte & 0xff).toString(16)).slice(-2);
  }).join('');
}

function construct_byte_array(addr, nonce, session_id, amount) {
  let arr = [];
  nonce = nonce.toArray('be', 8);
  session_id = session_id.toArray('be', 4);
  amount = amount.toArray('le', 16); // amount is le encoded
  arr.push(...addr, ...nonce, ...session_id, ...amount);
  return arr;
}

async function claimPayment(sender, receiver, nonceNum, sessionIdNum, amount) {
  let api = await getApi();
  let nonce = new BN(nonceNum);
  let sessionId = new BN(sessionIdNum);
  let amt = new BN(amount).mul(DPR);
  let res = construct_byte_array(receiver.publicKey, nonce, sessionId, amt);
  let msg = blake2AsU8a(res);
  let sig = sender.sign(msg);
  console.log(`ClaimPayment call: nonce: ${nonce}, session_id: ${sessionId}, deduct_amount: ${amount}`);
  const unsub = await api.tx.micropayment.claimPayment(sender.address, sessionId, amt, toHexString(sig))
    .signAndSend(receiver, {
      nonce: -1
    }, ({
      status
    }) => {
      if (status.isFinalized) {
        unsub();
      }
    });
}

async function closeExpiredChannels(sender) {
  let api = await getApi();
  return new Promise(async (resolve, reject) => {
    const unsub = await api.tx.micropayment.closeExpiredChannels().signAndSend(sender, {
      nonce: -1
    }, ({
      status,
      events
    }) => {
      if (status.isInBlock || status.isFinalized) {
        events
          .forEach((event) => {
            const result = event.event.data[0]
            // Now we look to see if the extrinsic was actually successful or not...
            if (result.isError) {
              let error = result.asError;
              if (error.isModule) {
                // for module errors, we have the section indexed, lookup
                const decoded = api.registry.findMetaError(error.asModule);
                const {
                  docs,
                  name,
                  section
                } = decoded;
                console.log(`${section}.${name}: ${docs.join(' ')}`);
                reject(`${section}.${name}: ${docs.join(' ')}`)
              } else {
                // Other, CannotLookup, BadOrigin, no extra info
                console.log(error.toString());
                reject(`${section}.${name}: ${docs.join(' ')}`)
              }
            }
          });
      
        const resultinfo = {
          result : events[0].event.index,
          sender : events[0].event.data[0],
          receiver: events[0].event.data[1]
        };

        unsub();
        resolve(resultinfo)
      }
    });
  })
}

// convert number in the smallest unit to DPR unit
function toDPR(amt) {
  return new BN(amt).div(DPR);
}

// convert number in DPR unit to number in the smallest unit of the currency.
function fromDPR(amt) {
  return new BN(amt).mul(DPR);
}

function sleep(time) {
  return new Promise(resolve => setTimeout(resolve, time))
}

module.exports = {
  getApi,
  openChannel,
  claimPayment,
  toDPR,
  fromDPR,
  sleep,
  closeExpiredChannels
}