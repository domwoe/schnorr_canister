# Schnorr Signature Canister

The purpose of the canister is to act as a developer preview for the Schnorr threshold signing API of the Internet Computer that is currently under development. The interface is compliant (with the exception that the final API will require attached cycles) with the [current draft for the Management Canister API](https://github.com/dfinity/interface-spec/pull/288) and will be updated as the API is updated.

If you are interested in building applications using threshold Schnorr you can get started today, and switch to the Management Canister API as soon as it is available. For ideas on what to build check out the [forum post](https://forum.dfinity.org/t/threshold-schnorr-facilitating-brc-20-trading-solana-integration-certificate-signing-and-more/28993) and the [Buidl on Bitcoin RFP](https://github.com/dfinity/grant-rfps/issues/58). 

For general updates on the development, please refer to the following [forum post](https://forum.dfinity.org/t/threshold-schnorr-facilitating-brc-20-trading-solana-integration-certificate-signing-and-more/28993).

DO NOT USE IN PRODUCTION! KEY MATERIAL COULD BE ACCESSED BY NODE PROVIDERS AND WE DON'T GUARANTEE THAT THIS CANISTER WILL STAY AVAILABLE. SEEDS ARE STORED IN STABLE MEMORY BUT WE DON'T GUARANTEE THAT KEYS MIGHT CHANGE DUE TO REINSTALLATION OF THE CANISTER.

## Supported Algorithms

- BIP340 (secp256k1) used for Bitcoin Taproot
- Ed25519 used in Solana, Cardano, Polkaddot, and others. Furthermore, it is approved by NIST and widely used in Web2.

## Add the canister to your project

Add the following to your `dfx.json` config file:

```json
{
  "canisters": {
    "schnorr_canister": {
      "type": "custom",
      "candid": "https://github.com/domwoe/schnorr_canister/releases/latest/download/schnorr_canister.did",
      "wasm": "https://github.com/domwoe/schnorr_canister/releases/latest/download/schnorr_canister.wasm.gz",
      "remote": {
        "id": {
          "ic": "6fwhw-fyaaa-aaaap-qb7ua-cai",
          "playground": "6fwhw-fyaaa-aaaap-qb7ua-cai"
        }
      }
    }
  }
}
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --clean --background

# Deploys your canisters to the replica and generates your candid interface
./scripts/deploy.sh
```

Once the job is completed, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

### Testing

We use [PocketIC](https://github.com/dfinity/pocketic) for integration testing. Please make sure to have it installed and the `POCKET_IC_BIN` environment variable set to the path of the `pocket-ic` binary.

You can run the tests with the following command:

```sh
./scripts/test.sh
```

## Deployment on the Internet Computer

The canister is deployed to `6fwhw-fyaaa-aaaap-qb7ua-cai`. 

You can check the Canid UI at [`https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=6fwhw-fyaaa-aaaap-qb7ua-cai`](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=6fwhw-fyaaa-aaaap-qb7ua-cai).

You can also see how many signatures have been generated at [https://6fwhw-fyaaa-aaaap-qb7ua-cai.raw.icp0.io/](https://6fwhw-fyaaa-aaaap-qb7ua-cai.raw.icp0.io/).

### Interact with Blast Playground

You can interact with the canister using the [Blast Playground](https://jglts-daaaa-aaaai-qnpma-cai.ic0.app/831.de93c1521f2395ef78586691ca27d4d3a0a937ebd0ffa442a1479769).

## To Do

- [ ] Potentially add cycles payments for creating signatures.


## Credits

This canister is monitored by [CycleOps](https://cycleops.dev).



