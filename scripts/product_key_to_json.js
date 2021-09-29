const { mnemonicGenerate, cryptoWaitReady } = require('@polkadot/util-crypto');
const { Keyring }  = require('@polkadot/keyring');

var fs = require('fs');

async function productKeyToJson() {
  await cryptoWaitReady();
  // create a keyring with some non-default values specified
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });

  const keys = {}
  const keyPareArray = [];

  
  for (let index = 0; index < 100; index++) {
    const mnemonic = mnemonicGenerate();
    // create & add the pair to the keyring with the type and some additional
    // metadata specified
    const pair = keyring.addFromUri(mnemonic, { name: 'first pair' });

    // the pair has been added to our keyring
    // console.log(keyring.pairs.length, 'pairs available');

    // log the name & address (the latter encoded with the ss58Format)
    // console.log(pair.meta.name, 'has address', pair.address);
    // console.log(pair.meta.name, 'has mnemonic', mnemonic);
    
    keyPareArray.push({address: pair.address, mnemonic: mnemonic});
  }
  //console.log(keyPareArray);


  keys['initPairs'] = keyPareArray;
  //console.log(keys);

  var initPairsPath = './initPairs.json';
  fs.writeFile(initPairsPath, JSON.stringify(keys, null, 4), function(err) {
    if(err) {
        console.log(err);
    } else {
        console.log("JSON saved to " + initPairsPath);
    }
  });
}

productKeyToJson()