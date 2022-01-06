window.BENCHMARK_DATA = {
  "lastUpdate": 1641464131452,
  "repoUrl": "https://github.com/deeper-chain/deeper-chain",
  "entries": {
    "Deeper Chain Benchmark": [
      {
        "commit": {
          "author": {
            "name": "deeper-chain",
            "username": "deeper-chain"
          },
          "committer": {
            "name": "deeper-chain",
            "username": "deeper-chain"
          },
          "id": "4bad72282fe11b7beec2c36799ec901ef406285e",
          "message": "fix bench error",
          "timestamp": "2022-01-05T01:19:07Z",
          "url": "https://github.com/deeper-chain/deeper-chain/pull/158/commits/4bad72282fe11b7beec2c36799ec901ef406285e"
        },
        "date": 1641435056653,
        "tool": "cargo",
        "benches": [
          {
            "name": "execute blocks/native",
            "value": 4287835,
            "range": "± 967314",
            "unit": "ns/iter"
          },
          {
            "name": "execute blocks/Wasm(Interpreted)",
            "value": 81747980,
            "range": "± 16546094",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joey.xie@deeper.network",
            "name": "Joey",
            "username": "xcaptain"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b7447dc4dbebf1e2037d3d55ddb073b36d7160ab",
          "message": "remove CheckedSignature (#158)\n\nadd benchmark ci",
          "timestamp": "2022-01-06T17:33:07+08:00",
          "tree_id": "7171858efc2ab42574c627b643ebcc25bf494e8c",
          "url": "https://github.com/deeper-chain/deeper-chain/commit/b7447dc4dbebf1e2037d3d55ddb073b36d7160ab"
        },
        "date": 1641464131081,
        "tool": "cargo",
        "benches": [
          {
            "name": "execute blocks/native",
            "value": 4635553,
            "range": "± 1074550",
            "unit": "ns/iter"
          },
          {
            "name": "execute blocks/Wasm(Interpreted)",
            "value": 84309045,
            "range": "± 15353022",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}