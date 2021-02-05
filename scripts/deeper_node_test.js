// Import the API, Keyring and some utility functions
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

async function getApiInstance() {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: {
      "Balance": "u128",
      "Timestamp": "Moment",
      "BlockNumber": "u32",
      "IpV4": "Vec<u8>",
      "CountryRegion": "Vec<u8>",
      "Duration": "u8",
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
      "CreditDelegateInfo": {
        "delegator": "AccountId",
        "score": "u64",
        "validators": "Vec<AccountId>"
      }
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

async function registerServer(api, signer, duration, test, expected) {
  const unsub = await api.tx.deeperNode
    .registerServer(duration)
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

async function unregisterServer(api, signer, test, expected) {
  const unsub = await api.tx.deeperNode
    .unregisterServer()
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
  let countryStr = info.country.toString('16');
  let country_code = '';
  for (let j = 1; j < countryStr.length / 2; j++) {
    country_code += String.fromCharCode(parseInt(countryStr.substring(2 * j, 2 * j + 2), 16));
  }
  if (country_code == expected)
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
  let country = "US";
  let ip = "192.168.100.113";
  let duration = 1;
  let invalid_duration = 10;
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
  await registerServer(api, signer, duration, 1, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #2, device is not registered with invalid duration
  console.log("Running test #2");
  await registerServer(api, signer, invalid_duration, 2, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #3, device is not registered
  console.log("Running test #3");
  await unregisterServer(api, signer, 3, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #4, device is not registered
  console.log("Running test #4");
  await unregisterDevice(api, signer, 4, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // error: #5, invalid country code
  console.log("Running test #5");
  await registerDevice(api, signer, ip, "ZZ", 5, "Failed");

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
  await unregisterServer(api, signer, 8, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // error: #9, register device as a server with invalid duration
  console.log("Running test #9");
  await registerServer(api, signer, invalid_duration, 9, "Failed");

  await new Promise(r => setTimeout(r, 30000));

  // success: #10, register device as a server
  console.log("Running test #10");
  await registerServer(api, signer, duration, 10, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #11, Alice is in the server list
  console.log("Running test #11");
  await getServersByCountry(api, country, 11, aliceAcct);

  await new Promise(r => setTimeout(r, 30000));

  // sucesss: #12, country code match
  console.log("Running test #12");
  await getDeviceInfo(api, aliceAcct, 12, country);

  await new Promise(r => setTimeout(r, 30000));

  // success: #13, ip match
  console.log("Running test #13");
  await getServerIP(api, aliceAcct, 13, ip);

  await new Promise(r => setTimeout(r, 30000));

  // success: #14, remove device from server list
  console.log("Running test #14");
  await unregisterServer(api, signer, 14, "Success");

  await new Promise(r => setTimeout(r, 30000));

  // success: #15, remove device from registration list
  console.log("Running test #15");
  await unregisterDevice(api, signer, 15, "Success");

  await new Promise(r => setTimeout(r, 30000));
}

unit_test().catch(console.error).finally(() => process.exit());
