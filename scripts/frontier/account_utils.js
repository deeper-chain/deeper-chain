const Web3 = require("web3");
const PrivateKeyProvider = require('truffle-privatekey-provider')

//const eth_private_key = 'Your eth account private key';
//const eth_address = 'Your eth account';

//const sub_address = 'Your substrate account';

const alice_addr = '0xd43593c715fdd31c61141abd04a99fd6822c8558';
const bob_addr = '0x8eaf04151687736326c9fea17e25fc5287613693';


async function test() {
    const provider = new PrivateKeyProvider(eth_private_key, "http://localhost:9933");
    const web3 = new Web3(provider);

    const sig = await web3.eth.personal.sign('deeper evm:' + sub_address, eth_address);
    //const sig = await web3.eth.sign(message, eth_address);
    console.log(sig);

    //await sendRawTransaction(web3, eth_address, eth_private_key, bob_addr).then();
}

test().then(() => {
    process.exit();
});

async function sendRawTransaction(web3, from, private_key, to) {
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
