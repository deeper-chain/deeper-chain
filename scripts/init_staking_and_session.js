const { mnemonicGenerate, cryptoWaitReady } = require('@polkadot/util-crypto');
const { Keyring }  = require('@polkadot/keyring');
const { stringToU8a, u8aToHex }   = require( '@polkadot/util');

const data = require('.//initPairs.json');
const arr = Object.values(data.initPairs)

//@@@@1 
//stash and SR25519
//“balance”
async function productBalance() {
  await cryptoWaitReady();
  console.log("@@@@@productBalance");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      
      console.log("          [");
      console.log("            \"" + pairSR25519.address + "\",");
      console.log("            10000000000000000000000000");
      console.log("          ],");
  });

  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      console.log("          [");
      console.log("            \"" + pairSR25519Stash.address + "\",");
      console.log("            10000000000000000000000000");
      console.log("          ],");
  });
}

//@@@@2
//[stash, SR25519]
async function productuserCreditData() {
  await cryptoWaitReady();
  console.log("@@@@@productuserCreditData");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      
      console.log("      [");
      console.log("        \"" + pairSR25519Stash.address + "\",");
      console.log("        {");
      console.log("          \"campaign_id\": 0,");
      console.log("          \"credit\": 100,");
      console.log("          \"initial_credit_level\": \"One\",");
      console.log("          \"rank_in_initial_credit_level\": 1,");
      console.log("          \"number_of_referees\": 1,");
      console.log("          \"current_credit_level\": \"One\",");
      console.log("          \"reward_eras\": 100");
      console.log("        }");
      console.log("      ],");
  });
}

//@@@@3
//[stash, SR25519]
async function productInvulnerables() {
  await cryptoWaitReady();
  console.log("@@@@@productInvulnerables");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      console.log("          \"" + pairSR25519Stash.address + "\",");
  });
}


//@@@@4
//[stash, SR25519]
async function productStaking() {
  await cryptoWaitReady();
  console.log("@@@@@productStaking");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      
      console.log("          [");
      console.log("            \"" + pairSR25519Stash.address + "\",");
      console.log("            \"" + pairSR25519.address + "\",");
      console.log("            20000000000000000000000,");
      console.log("            \"Validator\"");
      console.log("          ],");
  });
}

//@@@@5
async function productSessionKey() {
  await cryptoWaitReady();
  console.log("@@@@@productSessionKey");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      console.log("      [");
      console.log("        \"" + pairSR25519Stash.address + "\",");
      console.log("        \"" + pairSR25519Stash.address + "\",");
      console.log("        {");
      console.log("          \"grandpa\": \"" + pairED25519.address + "\",");
      console.log("          \"babe\": \"" + pairSR25519.address + "\",");
      console.log("          \"im_online\": \"" + pairSR25519.address + "\",");
      console.log("          \"authority_discovery\": \"" + pairSR25519.address + "\"");
      console.log("        }");
      console.log("      ],");

  });
}


//@@@@6
//[stash, SR25519]
async function productPalletCollectiveInstance2() {
  await cryptoWaitReady();
  console.log("@@@@@productPalletCollectiveInstance2");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });
      
      console.log("          \"" + pairSR25519Stash.address + "\",");
  });
}


//@@@@@7
//[stash, SR25519]
async function productPalletElectionsPhragmen() {
  await cryptoWaitReady();
  console.log("@@@@@productPalletElectionsPhragmen");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });

      console.log("     [");
      console.log("        \"" + pairSR25519Stash.address + "\",");
      console.log("        20000000000000000000000");
      console.log("      ],");
  });
}

//@@@@@8
//[stash, SR25519]
async function productPalletSociety() {
  await cryptoWaitReady();
  console.log("@@@@@productPalletSociety");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  console.log(",");
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });
      
      console.log("          \"" + pairSR25519Stash.address + "\",");
  });
}

//@@@@@9
//[stash, SR25519]
async function productInsertShell() {
  await cryptoWaitReady();
  console.log("@@@@@productInsertShell");

  // create a keyring with some non-default values specified
  const keyringSR25519Stash = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringSR25519 = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const keyringED25519 = new Keyring({ type: 'ed25519', ss58Format: 42 });

  let index = 0;
  arr.forEach(element => {
      const pairSR25519Stash = keyringSR25519Stash.addFromUri(element.mnemonic + "//stash", { name: 'first sr25519 pair' });
      const pairSR25519 = keyringSR25519.addFromUri(element.mnemonic, { name: 'first sr25519 pair' });
      const pairED25519 = keyringED25519.addFromUri(element.mnemonic, { name: 'first ed25519 pair' });
      
      console.log("curl -H 'Content-Type: application/json' --data '{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"author_insertKey\", \"params\": [\"babe\", \"" + element.mnemonic + "\", \"" + u8aToHex(pairSR25519.publicKey)+ "\"]}' http://localhost:" + (19933 + index));
      console.log("curl -H 'Content-Type: application/json' --data '{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"author_insertKey\", \"params\": [\"gran\", \"" + element.mnemonic + "\", \"" + u8aToHex(pairED25519.publicKey)+ "\"]}' http://localhost:" + (19933 + index));
      index++;
  });
}

productBalance()
productuserCreditData()
productInvulnerables()
productStaking()
productSessionKey()
productPalletCollectiveInstance2()
productPalletElectionsPhragmen()
productPalletSociety()

productInsertShell()
