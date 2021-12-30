const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

//const private_key = 'YOUR-PRIVATE-KEY-HERE';
//const my_address = 'PUBLIC-ADDRESS-OF-PK-HERE';
const private_key = '0xf64400f20ffd595d9e66dc4addf6af30530067ea23e4d5534c80aafe4b14534a';
const my_address = '0x3d0e26278Cb3C2Bf17fa933C9E65Fd63973c0Be0';

const alice_addr = '0xd43593c715fdd31c61141abd04a99fd6822c8558';
const bob_addr = '0x8eaf04151687736326c9fea17e25fc5287613693';

async function test() {
    //web3.eth.defaultAccount = my_address;

    let alice_balance = await getBalance(alice_addr);
    let bob_balance = await getBalance(bob_addr);
    let my_balance = await getBalance(my_address);

    let protocol_version = await getProtocolVersion();
    if (protocol_version != 1) {
        console.error("protocol version is wrong: " + protocol_version);
    }

    let syncing = await isSyncing();

    let hash_rate = await getHashRate();
    if (hash_rate != 0) {
        console.error("hash rate is wrong: " + hash_rate);
    }

    let coinbas = await getCoinbase();

    let is_mining = await isMining();

    let chain_id = await getChainId();
    if (chain_id != 43) {
        console.error("chain id is wrong: " + chain_id);
    }

    let gas_price = await getGasPrice();

    let accounts = await getAccounts();

    let block_num = await getBlockNum();

    let my_tx_cnt = await getTransactionCount(my_address);
    let my_tx_hash = await sendRawTransaction(my_address, private_key, bob_addr);
    let my_tx_cnt_1 = await getTransactionCount(my_address);

    if (my_tx_cnt_1 - my_tx_cnt != 1) {
        console.error("transaction cnt is not correct!");
    }

    let my_tx = await getTransactionByHash(my_tx_hash);
    let my_receipt = await getTransactionReceipt(my_tx_hash);

    let my_block = my_receipt.blockNumber;

    let block = await getBlockByNumber(my_block);
    let block1 = await getBlockByHash(block.hash);

    if (JSON.stringify(block) != JSON.stringify(block1)) {
        console.error("blocks are not the same!");
    }

    let tx_cnt1 = await getBlockTransactionCountByHash(block.hash);
    let tx_cnt2 = await getBlockTransactionCountByNumber(my_block);

    let uncle_cnt = await getUncleCountByBlockHash(block.hash);
    let uncle_cnt1 = await getUncleCountByBlockNumber(my_block);
    if (uncle_cnt != 0 || uncle_cnt1 != 0) {
        console.error("uncle cnt/cnt1 is not correct!");
    }

    let tx1 = await getTransactionByBlockHashAndIndex(block.hash, 0);
    let tx2 = await getTransactionByBlockNumberAndIndex(my_block, 0);

    let uncle = await getUncleByBlockHashAndIndex(block.hash, 0);
    let uncle1 = await getUncleByBlockNumberAndIndex(my_block, 0);
    if (uncle != null || uncle1 != null) {
        console.error("uncle/uncle1 is not correct!");
    }

    let contract_addr = await deployContract(my_address, private_key);

    let storage = await getStorageAt(contract_addr);
    let code = await getCode(contract_addr);

    let alice_token_balance = await balanceOf(alice_addr, contract_addr);
    let my_token_balance = await balanceOf(my_address, contract_addr);

    let tx_cnt = 1000;
    let tx = getTxAbi(alice_addr, tx_cnt, alice_addr);
    let gas = await estimateGas(my_address, contract_addr, tx);

    await transferToken(my_address, contract_addr, tx);

    let alice_token_balance_1 = await balanceOf(alice_addr, contract_addr);
    let my_token_balance_1 = await balanceOf(my_address, contract_addr);

    if (alice_token_balance_1 - alice_token_balance != tx_cnt) {
        console.error("token transfer failed!!!");
    }

    //let alice_token_balance1 = await call(contract_addr, "0x70a08231000000000000000000000000d43593c715fdd31c61141abd04a99fd6822c8558");

    let submit_ret = await submitWork(
        "0x0000000000000001",
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "0xD1FE5700000000000000000000000000D1FE5700000000000000000000000000"
    );
    if (submit_ret != false) {
	console.error("submitWork should return false");
    }

    let work = await getWork();

    /*
    let log = await getLogs(tx);
    */
}

test().then(() => {
    console.log("test is over");
    process.exit();
});


async function getBalance(address) {
    const balance = await web3.eth.getBalance(address);
    console.log("balance: " + balance);
    return balance;
}

async function getProtocolVersion() {
    const version = await web3.eth.getProtocolVersion();
    console.log("version: " + version);
    return version;
}

async function isSyncing() {
    const syncing = await web3.eth.isSyncing();
    console.log("syncing: " + syncing);
    return syncing;
}

async function getHashRate() {
    const hashrate = await web3.eth.getHashrate();
    console.log("hashrate: " + hashrate);
    return hashrate;
}

async function getCoinbase() {
    const author = await web3.eth.getCoinbase();
    console.log("coinbase: " + author);
    return author;
}

async function isMining() {
    const mining = await web3.eth.isMining();
    console.log("mining: " + mining);
    return mining;
}

async function getChainId() {
    const chain_id = await web3.eth.getChainId();
    console.log("chainId: " + chain_id);
    return chain_id;
}

async function getGasPrice() {
    const gas_price = await web3.eth.getGasPrice();
    console.log("gasPrice: " + gas_price);
    return gas_price;
}

async function getAccounts() {
    const accounts = await web3.eth.getAccounts();
    console.log("acounts: " + accounts);
    return accounts;
}

async function getBlockNum() {
    const block_num = await web3.eth.getBlockNumber();
    console.log("blockNumber: " + block_num);
    return block_num;
}

async function sendRawTransaction(from, private_key, to) {
    const createTransaction = await web3.eth.accounts.signTransaction(
	{
	    gas: 21000,
	    to: to,
	    value: web3.utils.toWei('1', 'ether'),
	},
	private_key
    );

    const createReceipt = await web3.eth.sendSignedTransaction(
	createTransaction.rawTransaction
    );
    console.log(
	`Transaction successful with hash: ${createReceipt.transactionHash}`
    );

    return createReceipt.transactionHash;
};

async function getBlockByHash(hash, full_transaction=false) {
    const block = await web3.eth.getBlock(hash, full_transaction);
    console.log("block by hash: " + JSON.stringify(block));
    return block;
}

async function getBlockByNumber(num, full_transaction=false) {
    const block = await web3.eth.getBlock(num, full_transaction);
    console.log("block by num: " + JSON.stringify(block));
    return block;
}

async function getBlockTransactionCountByHash(hash) {
    const cnt = await web3.eth.getBlockTransactionCount(hash);
    console.log("transaction count by hash: " + cnt);
    return cnt;
}

async function getBlockTransactionCountByNumber(num) {
    const cnt = await web3.eth.getBlockTransactionCount(num);
    console.log("transaction count by num: " + cnt);
    return cnt;
}

async function getUncleCountByBlockHash(hash) {
    const cnt = await web3.eth.getBlockUncleCount(hash);
    console.log("uncle count by hash: " + cnt);
    return cnt;
}

async function getUncleCountByBlockNumber(num) {
    const cnt = await web3.eth.getBlockUncleCount(num);
    console.log("uncle count by num: " + cnt);
    return cnt;
}

async function getTransactionCount(address, block=web3.eth.defaultBlock) {
    const cnt = await web3.eth.getTransactionCount(address, block);
    console.log("transaction count: " + cnt);
    return cnt;
}

async function getTransactionByHash(hash) {
    const tx = await web3.eth.getTransaction(hash);
    console.log("transaction: " + JSON.stringify(tx));
    return tx;
}

async function getTransactionReceipt(hash) {
    const receipt = await web3.eth.getTransactionReceipt(hash);
    console.log("receipt: " + JSON.stringify(receipt));
    return receipt;
}

async function getTransactionByBlockHashAndIndex(hash, idx) {
    const tx = await web3.eth.getTransactionFromBlock(hash, idx);
    console.log("transaction by hash: " + JSON.stringify(tx));
    return tx;
}

async function getTransactionByBlockNumberAndIndex(num, idx) {
    const tx = await web3.eth.getTransactionFromBlock(num, idx);
    console.log("transaction by num: " + JSON.stringify(tx));
    return tx;
}

async function getUncleByBlockHashAndIndex(hash, idx) {
    const uncle = await web3.eth.getUncle(hash, idx);
    console.log("uncle by hash: " + JSON.stringify(uncle));
    return uncle;
}

async function getUncleByBlockNumberAndIndex(num, idx) {
    const uncle = await web3.eth.getUncle(num, idx);
    console.log("uncle by num: " + JSON.stringify(uncle));
    return uncle;
}

async function getStorageAt(address, position=0) {
    const storage = await web3.eth.getStorageAt(address, position);
    console.log("stoarge: " + storage);
    return storage;
}

async function getCode(address, defaultBlock=web3.eth.defaultBlock) {
    const code = await web3.eth.getCode(address, defaultBlock);
    console.log("code: " + code);
    return code;
}

async function balanceOf(address, contract_addr) {
    const abi = require('./my_token.json');
    let contract = new web3.eth.Contract(abi, contract_addr);
    const balance = await contract.methods.balanceOf(address).call((err, result) => { return result; });
    console.log("balance: " + balance);
    return balance;
}

async function call(to, data) {
    let result = await web3.eth.call({
        to: to, // contract address
        data: data
    });
    console.log("call return: " + result);
    return result;
}

async function estimateGas(from, to, data) {
    let gas = await web3.eth.estimateGas({
        from: from,
        to: to,
        data: data
    });
    console.log("estimate gas: " + gas);
    return gas;
}

async function submitWork(nonce, powHash, digest) {
    let result = await web3.eth.submitWork(
        nonce,
        powHash,
        digest,
    );
    console.log("submit work: " + result);
    return result;
}

async function getWork() {
    const work = await web3.eth.getWork();
    console.log("work: " + work);
    return work;
}

// from https://github.com/paritytech/frontier/blob/master/template/examples/contract-erc20/truffle/contracts/MyToken.json#L259
const erc20_bytecode = '0x608060405234801561001057600080fd5b50610041337fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff61004660201b60201c565b610291565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156100e9576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601f8152602001807f45524332303a206d696e7420746f20746865207a65726f20616464726573730081525060200191505060405180910390fd5b6101028160025461020960201b610c7c1790919060201c565b60028190555061015d816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205461020960201b610c7c1790919060201c565b6000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040518082815260200191505060405180910390a35050565b600080828401905083811015610287576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601b8152602001807f536166654d6174683a206164646974696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b8091505092915050565b610e3a806102a06000396000f3fe608060405234801561001057600080fd5b50600436106100885760003560e01c806370a082311161005b57806370a08231146101fd578063a457c2d714610255578063a9059cbb146102bb578063dd62ed3e1461032157610088565b8063095ea7b31461008d57806318160ddd146100f357806323b872dd146101115780633950935114610197575b600080fd5b6100d9600480360360408110156100a357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610399565b604051808215151515815260200191505060405180910390f35b6100fb6103b7565b6040518082815260200191505060405180910390f35b61017d6004803603606081101561012757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506103c1565b604051808215151515815260200191505060405180910390f35b6101e3600480360360408110156101ad57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019092919050505061049a565b604051808215151515815260200191505060405180910390f35b61023f6004803603602081101561021357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061054d565b6040518082815260200191505060405180910390f35b6102a16004803603604081101561026b57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610595565b604051808215151515815260200191505060405180910390f35b610307600480360360408110156102d157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610662565b604051808215151515815260200191505060405180910390f35b6103836004803603604081101561033757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050610680565b6040518082815260200191505060405180910390f35b60006103ad6103a6610707565b848461070f565b6001905092915050565b6000600254905090565b60006103ce848484610906565b61048f846103da610707565b61048a85604051806060016040528060288152602001610d7060289139600160008b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000610440610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b600190509392505050565b60006105436104a7610707565b8461053e85600160006104b8610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008973ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b61070f565b6001905092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60006106586105a2610707565b8461065385604051806060016040528060258152602001610de160259139600160006105cc610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008a73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b6001905092915050565b600061067661066f610707565b8484610906565b6001905092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610795576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526024815260200180610dbd6024913960400191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141561081b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526022815260200180610d286022913960400191505060405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040518082815260200191505060405180910390a3505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561098c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526025815260200180610d986025913960400191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610a12576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526023815260200180610d056023913960400191505060405180910390fd5b610a7d81604051806060016040528060268152602001610d4a602691396000808773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b6000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550610b10816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b6000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040518082815260200191505060405180910390a3505050565b6000838311158290610c69576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825283818151815260200191508051906020019080838360005b83811015610c2e578082015181840152602081019050610c13565b50505050905090810190601f168015610c5b5780820380516001836020036101000a031916815260200191505b509250505060405180910390fd5b5060008385039050809150509392505050565b600080828401905083811015610cfa576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601b8152602001807f536166654d6174683a206164646974696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b809150509291505056fe45524332303a207472616e7366657220746f20746865207a65726f206164647265737345524332303a20617070726f766520746f20746865207a65726f206164647265737345524332303a207472616e7366657220616d6f756e7420657863656564732062616c616e636545524332303a207472616e7366657220616d6f756e74206578636565647320616c6c6f77616e636545524332303a207472616e736665722066726f6d20746865207a65726f206164647265737345524332303a20617070726f76652066726f6d20746865207a65726f206164647265737345524332303a2064656372656173656420616c6c6f77616e63652062656c6f77207a65726fa265627a7a72315820c7a5ffabf642bda14700b2de42f8c57b36621af020441df825de45fd2b3e1c5c64736f6c63430005100032';

async function deployContract(from, private_key) {
    const createTransaction = await web3.eth.accounts.signTransaction(
        {
            data: erc20_bytecode,
            from: from,
            gas: 2000000,
        },
        private_key
    );

    const createReceipt = await web3.eth.sendSignedTransaction(
        createTransaction.rawTransaction
    );
    console.log(
        `Transaction successful with address: ${createReceipt.contractAddress}`
    );

    return createReceipt.contractAddress;
}

function getTxAbi(to, amount, contract_addr) {
    const abi = require('./my_token.json');
    let contract = new web3.eth.Contract(abi, contract_addr);
    let data = contract.methods.transfer(to, amount).encodeABI();
    console.log("tx abi: " + data);
    return data;
}

async function transferToken(from, to, data) {
    const createTransaction = await web3.eth.accounts.signTransaction(
        {
            data: data,
            from: from,
            gas: 2000000,
            to: to,
        },
        private_key
    );

    const createReceipt = await web3.eth.sendSignedTransaction(
        createTransaction.rawTransaction
    );
    console.log(
        `Transaction successful with hash: ${createReceipt.transactionHash}`
    );

    return createReceipt.transactionHash;
}

// Not supported
async function submitHashrate() {
}

/*
async function getLogs(filter) {
    const log = await web3.eth.getPastLogs(filter);
    console.log("log: " + log);
    return log;
}

eth_newFilter
eth_newBlockFilter
eth_newPendingTransactionFilter
eth_getFilterChanges
eth_getFilterLogs
eth_uninstallFilter

eth_subscribe
eth_unsubscribe
*/
