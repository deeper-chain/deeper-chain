const { mnemonicGenerate, cryptoWaitReady } = require('@polkadot/util-crypto');
// const { stringToU8a, u8aToHex }   = require( '@polkadot/util');
const { ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const BN = require('bn.js');

// Substrate connection config
const WEB_SOCKET = 'ws://127.0.0.1:9944';

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


function getTps(arrayBlockInfo){
  //console.log(arrayBlockInfo);

  let beginTime = arrayBlockInfo[0].blockTime;
  let endTime = arrayBlockInfo[arrayBlockInfo.length - 1].blockTime;
  let diffSeconds = endTime.sub(beginTime).toNumber()/1000;

  let count = 0;
  arrayBlockInfo.forEach(element => {
    count += element.txsCount;
  });

  let tps = count/diffSeconds;
  return tps;
}

const main = async () => {
  const api = await connectSubstrate();

  let count = 0;
  var arrBlocks = [];

  const unsubscribe = await api.rpc.chain.subscribeFinalizedHeads(async header => {

    let signedBlock = await api.rpc.chain.getBlock(header.hash);

    // console.log(signedBlock.block.extrinsics[0].toHuman());
    // console.log(signedBlock.block.extrinsics.length);

    if (1 == signedBlock.block.extrinsics.length) {
      return;
    }
    let blockTime = new BN(signedBlock.block.extrinsics[0].toHuman().method.args[0].toString());

    let blockInfo = {
      blockNum: signedBlock.block.header.number,
      blockTime: blockTime,
      txsCount: signedBlock.block.extrinsics.length
    }

    arrBlocks.push(blockInfo);

    if (arrBlocks.length > 255) {
      arrBlocks.shift();
    }

    if (arrBlocks.length > 3) {
      let last3Array = arrBlocks.slice(Math.max(arrBlocks.length - 3, 0));
      console.log("####3 ", getTps(last3Array));

      let last5Array = arrBlocks.slice(Math.max(arrBlocks.length - 5, 0));
      console.log("####5 ", getTps(last5Array));

      let last10Array = arrBlocks.slice(Math.max(arrBlocks.length - 10, 0));
      console.log("####10 ", getTps(last10Array));

      console.log("####All", getTps(arrBlocks));

      console.log("\n");
    }

    if (++count === 256) {
      unsubscribe();
      process.exit(0);
    }

  });

}

main()
  .catch(err => {
    console.log('error occur:', err);
    process.exit(1);
  })