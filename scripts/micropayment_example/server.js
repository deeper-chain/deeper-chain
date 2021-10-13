const {
  Keyring
} = require('@polkadot/api');
const {
  getApi,
  sleep,
  claimPayment,
  toDPR
} = require('./utils')

const MINAMOUNT = 1

async function main() {
  console.log('start server')
  const api = await getApi();
  const keyring = new Keyring({
    type: 'sr25519'
  });
  const alice = keyring.addFromUri('//Alice');
  const bob = keyring.addFromUri('//Bob');
  const dave = keyring.addFromUri('//Dave');

  let micropayments = [
    {
      sender: alice,
      receiver: dave,
      senderName: 'Alice',
      receiverName: 'Dave'
    },
    {
      sender: bob,
      receiver: dave,
      senderName: 'Bob',
      receiverName: 'Dave'
    }
  ]

  let amountMap = {
    Alice: 0,
    Bob: 0
  }
  
  while (true) {
    // We design two cases:
    // clientA will continuously use the traffic and simulate a chargeback at 0.1 DPR per second; 
    // clientB will not use the traffic, so no chargeback
    // Simulate 0.1 DPR deduction per second for clientA using the delay function
    await sleep(1000)
    // Simulate traversal of all channels related to Dave
    for (let i = 0; i < micropayments.length; i++) {
      let micropayment = micropayments[i]
      let channelData = await api.query.micropayment.channel(micropayment.sender.address, micropayment.receiver.address);
      let balance = toDPR(channelData.balance.toString())/1
      let amount = amountMap[micropayment.senderName];

      // Dynamic display of current channel status
      console.log(`${micropayment.senderName} to ${micropayment.receiverName} channel balance: ${balance} costAmount: ${amount}`)
      // If the accumulated cost of a channel reaches MINAMOUNT, it will trigger a claimPayment call
      if (amount >= MINAMOUNT) {
        let sidOption = await api.query.micropayment.sessionId([micropayment.sender.address, micropayment.receiver.address]);
        let sid = parseInt(sidOption) + 1;
        await claimPayment(micropayment.sender, micropayment.receiver, channelData.nonce, sid, amount); 
        amountMap[micropayment.senderName] = 0
      }
      if (micropayment.senderName == 'Alice') {
        amountMap.Alice = (amountMap.Alice + 0.1).toFixed(6)/1
      }
    }    
  }
}

main()