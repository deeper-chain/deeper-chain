const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

web3.eth.extend({
    property: 'txpool',
    methods: [{
        name: 'content',
        call: 'txpool_content'
    },{
        name: 'inspect',
        call: 'txpool_inspect'
    },{
        name: 'status',
        call: 'txpool_status'
    }]
});

async function test() {
    let cnt = 0;
    while (1) {
        await getTxpoolStatus();
        await getTxpoolContent();
        await getTxpoolInspect();
        await new Promise(r => setTimeout(r, 2000));
        cnt += 1;
        if (cnt == 1000)
            break;
    }
}

test().then(() => {
    console.log("test is over");
    process.exit();
});

async function getTxpoolStatus() {
    const status = await web3.eth.txpool.status();
    let str = JSON.stringify(status);
    console.log("txpool status: " + str);
    return status;
}

async function getTxpoolContent() {
    const content = await web3.eth.txpool.content();
    let str = JSON.stringify(content);
    console.log("txpool content: " + str);
    return content;
}

async function getTxpoolInspect() {
    const inspect = await web3.eth.txpool.inspect();
    let str = JSON.stringify(inspect);
    console.log("txpool inspect: " + str);
    return inspect;
}
