const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

//const private_key = 'YOUR-PRIVATE-KEY-HERE';
//const my_address = 'PUBLIC-ADDRESS-OF-PK-HERE';

const alice_addr = '0xd43593c715fdd31c61141abd04a99fd6822c8558';
const bob_addr = '0x8eaf04151687736326c9fea17e25fc5287613693';

const contract_addr = '0x8a50db1e0f9452cfd91be8dc004ceb11cb08832f';

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

    let storage = await getStorageAt(contract_addr);
    let code = await getCode(contract_addr);

    let gas = await estimateGas(alice_addr, contract_addr, '0xa9059cbb0000000000000000000000008eaf04151687736326c9fea17e25fc528761369300000000000000000000000000000000000000000000000000000000000000dd');

    let alice_token_balance = await balanceOf(alice_addr);

    let alice_token_balance1 = await call(contract_addr, "0x70a08231000000000000000000000000d43593c715fdd31c61141abd04a99fd6822c8558");

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

async function balanceOf(address) {
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
