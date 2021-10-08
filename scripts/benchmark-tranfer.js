const { mnemonicGenerate, cryptoWaitReady } = require('@polkadot/util-crypto');
const { Keyring }  = require('@polkadot/keyring');
const { stringToU8a, u8aToHex }   = require( '@polkadot/util');
const { ApiPromise, WsProvider} = require('@polkadot/api');
// Substrate connection config
const WEB_SOCKET = 'ws://127.0.0.1:9944';

const data = require('.//initPairs.json');
const arr = Object.values(data.initPairs)

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));


const connectSubstrate = async () => {
  const wsProvider = new WsProvider(WEB_SOCKET);
  const api = await ApiPromise.create({ provider: wsProvider, types: {
    "Address": "MultiAddress",
    "LookupSource": "MultiAddress",
    "AccountInfo": "AccountInfoWithDualRefCount",
    "Balance": "u128",
    "Timestamp": "Moment",
    "BlockNumber": "u32",
    "IpV4": "Vec<u8>",
    "CountryRegion": "Vec<u8>",
    "DurationEras": "u8",
    "Node": {
        "account_id": "AccountId",
        "ipv4": "IpV4",
        "country": "CountryRegion",
        "expire": "BlockNumber"
    },
    "ChannelOf": {
        "sender": "AccountId",
        "receiver": "AccountId",
        "balance": "Balance",
        "nonce": "u64",
        "opened": "BlockNumber",
        "expiration": "BlockNumber"
    },
    "MemberId": "u64",
  "ProposalId": "u64",
  "Limits": {
    "max_tx_value": "u128",
    "day_max_limit": "u128",
    "day_max_limit_for_one_address": "u128",
    "max_pending_tx_limit": "u128",
    "min_tx_value": "u128"
  },
  "Status": {
    "_enum": [
      "Revoked",
      "Pending",
      "PauseTheBridge",
      "ResumeTheBridge",
      "UpdateValidatorSet",
      "UpdateLimits",
      "Deposit",
      "Withdraw",
      "Approved",
      "Canceled",
      "Confirmed"
    ]
  },
  "Kind": {
    "_enum": [
      "Transfer",
      "Limits",
      "Validator",
      "Bridge"
    ]
  },
  "TransferMessage": {
    "message_id": "H256",
    "eth_address": "H160",
    "substrate_address": "AccountId",
    "amount": "TokenBalance",
    "status": "Status",
    "action": "Status"
  },
  "LimitMessage": {
    "id": "H256",
    "limits": "Limits",
    "status": "Status"
  },
  "BridgeMessage": {
    "message_id": "H256",
    "account": "AccountId",
    "status": "Status",
    "action": "Status"
  },
  "ValidatorMessage": {
    "message_id": "H256",
    "quorum": "u64",
    "accounts": "Vec<AccountId>",
    "status": "Status",
    "action": "Status"
  },
  "BridgeTransfer": {
    "transfer_id": "ProposalId",
    "message_id": "H256",
    "open": "bool",
    "votes": "MemberId",
    "kind": "Kind"
  },
  "CreditLevel": {
    "_enum": [
      "Zero",
      "One",
      "Two",
      "Three",
      "Four",
      "Five",
      "Six",
      "Seven",
      "Eight"
    ]
  },
  "CampaignId": "u16",
  "CreditSetting": {
    "campaign_id": "CampaignId",
    "credit_level": "CreditLevel",
    "staking_balance": "Balance",
    "base_apy": "Percent",
    "bonus_apy": "Percent",
    "max_rank_with_bonus": "u32",
    "tax_rate": "Percent",
    "max_referees_with_rewards": "u8",
    "reward_per_referee": "Balance"
  },
  "CreditData": {
    "campaign_id": "CampaignId",
    "credit": "u64",
    "initial_credit_level": "CreditLevel",
    "rank_in_initial_credit_level": "u32",
    "number_of_referees": "u8",
    "current_credit_level": "CreditLevel",
    "reward_eras": "EraIndex"
  },
  "DelegatorData": {
    "delegator": "AccountId",
    "delegated_validators": "Vec<AccountId>",
    "unrewarded_since": "Option<EraIndex>",
    "delegating": "bool"
  },
  "EraIndex": "u32",
  "ValidatorData": {
    "delegators": "Vec<AccountId>",
    "elected_era": "EraIndex"
  },
  "RewardData": {
    "total_referee_reward": "Balance",
    "received_referee_reward": "Balance",
    "referee_reward": "Balance",
    "received_pocr_reward": "Balance",
    "poc_reward": "Balance"
  },
  "ValidatorPrefs": {
    "commission": "Perbill",
    "blocked": "bool"
  }
  } });
  return api;
};

const main = async () => {
  const api = await connectSubstrate();

  await cryptoWaitReady();


  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const tokeyPareArray = [];
  for (let index = 0; index < 1000; index++) {
    const mnemonic = mnemonicGenerate();
    const pair = keyring.addFromUri(mnemonic, { name: 'first pair' });
    tokeyPareArray.push({address: pair.address, mnemonic: mnemonic});
  }
  console.log("###tokeyPareArray done");

  let arrPairs = []
  arr.forEach(element => {
    const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
    const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
    arrPairs.push(pairSR25519);
  });

  arr.forEach(element => {
    const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
    const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
    arrPairs.push(pairSR25519);
  });

  // for (let i = 0; i < arrPairs.length; i++) {
  //   for (let j = 0; j < arrPairs.length; j++) {
  //     if (i === j) {
  //       continue;
  //     }

  //     const from = arrPairs[j];
  //     const to = arrPairs[i];

  //     let temptx = api.tx.balances
  //     .transfer(to.address, Date.now());
  //     //console.log("##### new tx", from.address, to.address, temptx.toJSON());

  //     const unsub = await temptx.signAndSend(from, (result) => {
  //                         //temptx.signAndSend(from, (result) => {
  //       //console.log(`Current status is ${result.status}`);

  //       if (result.status.isInBlock) {
  //         //console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
  //       } else if (result.status.isFinalized) {
  //         console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
  //         unsub();
  //       }
  //     });
  //   }
  // }

  for (let i = 0; i < tokeyPareArray.length; i++) {
    for (let j = 0; j < arrPairs.length; j++) {
      const from = arrPairs[j];
      const to = tokeyPareArray[i];

      let temptx = api.tx.balances
      .transfer(to.address, Date.now());
      console.log("##### new tx", from.address, to.address, temptx.toJSON());

      const unsub = await temptx.signAndSend(from, (result) => {
                          //temptx.signAndSend(from, (result) => {
        //console.log(`Current status is ${result.status}`);

        if (result.status.isInBlock) {
          //console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
          unsub();
        }
      });
    }
  }
}

main()
  .then(() => {
    console.log("successfully exited");
    process.exit(0);
  })
  .catch(err => {
    console.log('error occur:', err);
    process.exit(1);
  })