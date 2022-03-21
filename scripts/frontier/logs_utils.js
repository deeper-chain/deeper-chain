const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

web3.eth.extend({
    property: 'eth',
    methods: [{
        name: 'newFilter',
        call: 'eth_newFilter',
        params: 1,
    },{
        name: 'getFilterLogs',
        call: 'eth_getFilterLogs',
        params: 1,
    }]
});

// const my_address = "smart contract address";

let filter1 = {
    fromBlock: 0,
    address: my_address,
};

let filter2 = {
    fromBlock: 0,
    address: my_address,
};

async function test() {
    await getLogs(filter1);
    let filter_id = await newFilter(filter2);
    await getFilterLogs(parseInt(filter_id));
}

test().then(() => {
    console.log("test is over");
    process.exit();
});

async function getLogs(filter) {
    const log = await web3.eth.getPastLogs(filter);
    let str = JSON.stringify(log);
    console.log("log: " + str);
    return log;
}

async function newFilter(filter) {
    const id = await web3.eth.eth.newFilter(filter);
    console.log("log: " + parseInt(id));
    return parseInt(id);
}

async function getFilterLogs(filter) {
    const log = await web3.eth.eth.getFilterLogs(filter);
    let str = JSON.stringify(log);
    console.log("log: " + str);
    return log;
}
