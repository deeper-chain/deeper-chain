// Import the API, Keyring and some utility functions
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

async function getApiInstance() {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: {
      TokenBalance: "u64",
      Timestamp: "Moment",
      Node: {
        account_id: "AccountId",
        ipv4: "Vec<u8>",
        country: "u16"
      },
      ChannelOf: {
        sender: "AccountId",
        receiver: "AccountId",
        nonce: "u64",
        opened: "Timestamp",
        expiration: "Timestamp"
      },
      Erc20Token: {
        name: "Vec<u8>",
        ticker: "Vec<u8>",
        total_supply: "T",
      },
    },
  });
  return api;
}

async function registerDevice(api, signer, ip, country, test, expect) {
  const unsub = await api.tx.deeperNode
    .registerDevice(ip, country)
    .signAndSend(signer, ({ events = [], status }) => {
      if (status.isFinalized) {
        events.forEach(({ phase, event: { data, method, section } }) => {
          if (method == "ExtrinsicFailed")
            console.log("Test #" + test + ": registerDevice Failed, " + "expect " + expect);
          else if (method == "ExtrinsicSuccess")
            console.log("Test #" + test + ": registerDevice Success, " + "expect " + expect);
        });

        unsub();
      }
    });
}

async function unregisterDevice(api, signer, test, expect) {
  const unsub = await api.tx.deeperNode
    .unregisterDevice()
    .signAndSend(signer, ({ events = [], status }) => {
      if (status.isFinalized) {
        events.forEach(({ phase, event: { data, method, section } }) => {
          if (method == "ExtrinsicFailed" ) {
            if (test == 0)
              console.log("Init success!");
            else
              console.log("Test #" + test + ": unregisterDevice Failed, " + "expect " + expect);
          } else if (method == "ExtrinsicSuccess") {
            if (test == 0)
              console.log("Init success!");
            else
              console.log("Test #" + test + ": unregisterDevice Success, " + "expect " + expect);
          }
        });

        unsub();
      }
    });
}

async function registerServer(api, signer, country, test, expected) {
  const unsub = await api.tx.deeperNode
    .registerServer(country)
    .signAndSend(signer, ({ events = [], status }) => {
      if (status.isFinalized) {
        events.forEach(({ phase, event: { data, method, section } }) => {
          if (method == "ExtrinsicFailed")
            console.log("Test #" + test + ": registerServer Failed, " + "expect " + expected);
          else if (method == "ExtrinsicSuccess")
            console.log("Test #" + test + ": registerServer Success, " + "expect " + expected);
        });

        unsub();
      }
    });
}

async function unregisterServer(api, signer, country, test, expected) {
  const unsub = await api.tx.deeperNode
    .unregisterServer(country)
    .signAndSend(signer, ({ events = [], status }) => {
      if (status.isFinalized) {
        events.forEach(({ phase, event: { data, method, section } }) => {
          if (method == "ExtrinsicFailed")
            console.log("Test #" + test + ": unregisterServer Failed, " + "expect " + expected);
          else if (method == "ExtrinsicSuccess")
            console.log("Test #" + test + ": unregisterServer Success, " + "expect " + expected);
        });

        unsub();
      }
    });
}

async function getServersByCountry(api, country, test, expected) {
  const servers = await api.query.deeperNode
    .serversByCountry(country);
  if (servers.length == 0) {
    if (expected == null)
      console.log("Test #" + test + ": getServersByCountry returned correct result, Success");
    else
      console.log("Test #" + test + ": getServersByCountry has NO return, Failed");
    return;
  }

  let found = false;
  for (const server of servers) {
    const serverStr = server.toString('16');
    if (expected == serverStr) {
      console.log("Test #" + test + ": getServersByCountry returned correct result, Success");
      found = true;
    }
  }
  if (found == false)
    console.log("Test #" + test + ": getServersByCountry didn't return " + expected + ", Failed");
}

async function getDeviceInfo(api, accountId, test, expected) {
  const info = await api.query.deeperNode
    .deviceInfo(accountId);
  if (info.country.words[0] == expected)
    console.log("Test #" + test + ": getDeviceInfo, Success");
  else
    console.log("Test #" + test + ": getDeviceInfo didn't return " + expected + ", Failed");
}

async function getServerIP(api, accountId, test, expected) {
  const info = await api.query.deeperNode
    .deviceInfo(accountId);
  const ipStr = info.ipv4.toString('16');
  let ipAddr = "";
  for (i = 1; i < ipStr.length; i++) {
    ipAddr += String.fromCharCode(parseInt(ipStr.substring(2 * i, 2 * i + 2), 16));
  }

  if (ipAddr.includes(expected))
    console.log("Test #" + test + ": getServerIP, Success");
  else
    console.log("Test #" + test + ": getServerIP didn't return " + expected + ", Failed");
}

async function unit_test() {
  let country = 100;
  let ip = "192.168.100.113";
  const api = await getApiInstance();
  const keyring = new Keyring({ type: 'sr25519' });
  const signer = keyring.addFromUri('//Alice');
  const aliceAcct = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

  // init: remove device registration of Alice
  console.log("Running Init process");
  await unregisterDevice(api, signer, 0, "");

  await new Promise(r => setTimeout(r, 30000));

  // error: #1, device is not registered
  console.log("Running test #1");
  await registerServer(api, signer, country, 1, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #2, device is not registered and country
  console.log("Running test #2");
  await registerServer(api, signer, 2048, 2, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #3, device is not registered
  console.log("Running test #3");
  await unregisterServer(api, signer, 2048, 3, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #4, device is not registered
  console.log("Running test #4");
  await unregisterDevice(api, signer, 4, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #5, invalid country code
  console.log("Running test #5");
  await registerDevice(api, signer, ip, 2048, 5, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #6, invalid encrypted ip
  console.log("Running test #6");
  await registerDevice(api, signer, "1234567890".repeat(50), country, 6, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // success: #7, register device
  console.log("Running test #7");
  await registerDevice(api, signer, ip, country, 7, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #8, device is registered
  console.log("Running test #8");
  await unregisterServer(api, signer, country, 8, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #9, register device as a server
  console.log("Running test #9");
  await registerServer(api, signer, country, 9, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #10, Alice is in the server list
  console.log("Running test #10");
  await getServersByCountry(api, country, 10, aliceAcct);

  await new Promise(r => setTimeout(r, 30000));

  // sucesss: #11, country code match
  console.log("Running test #11");
  await getDeviceInfo(api, aliceAcct, 11, country);

  await new Promise(r => setTimeout(r, 30000));

  // success: #12, ip match
  console.log("Running test #12");
  await getServerIP(api, aliceAcct, 12, ip);

  await new Promise(r => setTimeout(r, 30000));

  // success: #13, remove device from server list
  console.log("Running test #13");
  await unregisterServer(api, signer, country, 13, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #14, remove device from registration list
  console.log("Running test #14");
  await unregisterDevice(api, signer, 14, "Success");

  await new Promise(r => setTimeout(r, 30000));
}

unit_test().catch(console.error).finally(() => process.exit());
