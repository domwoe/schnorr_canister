# Schnorr Signature Canister

Canister to create schnorr signatures on the Internet Computer.

DO NOT USE IN PRODUCTION! KEY MATERIAL COULD BE ACCESSED BY NODE PROVIDERS!

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
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
./deploy.sh
```

Once the job is completed, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

### Testing

We use [PocketIC](https://github.com/dfinity/pocketic) for integration testing. Please make sure to have it installed and the `POCKET_IC_BIN` environment variable set to the path of the `pocket-ic` binary.

You can run the tests with the following command:

```sh
cargo test
```

## Deployment on the Internet Computer

The canister is deployed to `6fwhw-fyaaa-aaaap-qb7ua-cai`. 

You can check the Canid UI at [`https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=6fwhw-fyaaa-aaaap-qb7ua-cai`](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=6fwhw-fyaaa-aaaap-qb7ua-cai).

You can also see how many signatures have been generated at [https://6fwhw-fyaaa-aaaap-qb7ua-cai.raw.icp0.io/](https://6fwhw-fyaaa-aaaap-qb7ua-cai.raw.icp0.io/).

### Interact with Blast Playground

You can interact with the canister using the [Blast Playground](https://jglts-daaaa-aaaai-qnpma-cai.ic0.app/717.bec4e0dbcb426b2b62247b6ff9c267261c6668caed7d84ef59120c07).

## To Do

- [ ] Potentially add cycles payments for creating signatures.


## Credits

This canister is monitored by [CycleOps](https://cycleops.dev).



