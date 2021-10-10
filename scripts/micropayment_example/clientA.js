const {
  Keyring
} = require('@polkadot/api');
const {
  getApi,
  fromDPR,
  toDPR,
  openChannel,
  sleep,
  closeExpiredChannels
} = require('./utils')

// The channel will lock 20 DPR
const lockAmount = 20;
// The channel will last 105 seconds
const duration = 20;

// alice to dave
async function main() {
  console.log('start clientA')
  const api = await getApi();
  const keyring = new Keyring({
    type: 'sr25519'
  });
  const alice = keyring.addFromUri('//Alice');
  const dave = keyring.addFromUri('//Dave');
  
  const addressMap = {
    [alice.address]: 'Alice',
    [dave.address]: 'Dave',
  }

  // initialize channel
  await openChannel(alice, dave, fromDPR(lockAmount), duration);

  // verity channel status
  let channelData = await api.query.micropayment.channel(alice.address, dave.address);
  let balance = toDPR(channelData.balance.toString())
  if (balance/1 == lockAmount) {
    console.log(`OpenChannel [${addressMap[channelData.sender]} → ${addressMap[channelData.receiver]}]; balance: ` + balance/1)
  }

  // Check for expired channels every 10 seconds
  while (true) {
    let closeHash = await closeExpiredChannels(alice);
    // 0x2501 means that a channel close message is received and output is printed
    if (closeHash.result == 0x2501) {
      console.log(`Expired channel [${addressMap[closeHash.sender]} → ${addressMap[closeHash.receiver]}] is closed`)
    }
    await sleep(10000)
  }
}

main()