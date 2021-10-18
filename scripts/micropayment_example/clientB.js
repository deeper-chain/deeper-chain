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

const lockAmount = 10;
const duration = 20;

// bob to dave
async function main() {
  console.log('start clientB')
  const api = await getApi();
  const keyring = new Keyring({
    type: 'sr25519'
  });
  const bob = keyring.addFromUri('//Bob');
  const dave = keyring.addFromUri('//Dave');

  const addressMap = {
    [bob.address]: 'Bob',
    [dave.address]: 'Dave',
  }

  // Initialize channel, with a delay of 1 second in order to have an interval with client A
  await sleep(1000)
  await openChannel(bob, dave, fromDPR(lockAmount), duration);

  // verity channel status
  let channelData = await api.query.micropayment.channel(bob.address, dave.address);
  let balance = toDPR(channelData.balance.toString())
  if (balance/1 == lockAmount) {
    console.log(`OpenChannel [${addressMap[channelData.sender]} → ${addressMap[channelData.receiver]}]; balance: ` + balance/1)
  }

  // Check for expired channels every 10 seconds
  while (true) {
    let closeHash = await closeExpiredChannels(bob);
    // 0x2501 means that a channel close message is received and output is printed
    if (closeHash.result == 0x2501) {
      console.log(`Expired channel [${addressMap[closeHash.sender]} → ${addressMap[closeHash.receiver]}] is closed`)
    }
    await sleep(10000)
  }
}

main()