# Schnorr Signature Canister

Canister to create schnorr signatures on the Internet Computer.

DO NOT USE IN PRODUCTION! KEY MATERIAL COULD BE ACCESSED BY NODE PROVIDERS!

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
./deploy.sh
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

## Deployment on the Internet Computer

The canister is deployed to `htvbm-vaaaa-aaaap-qb5kq-cai`. You can check the Canid UI at [`https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=htvbm-vaaaa-aaaap-qb5kq-cai`](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=htvbm-vaaaa-aaaap-qb5kq-cai).

You can also see how many signatures have been generated at [https://htvbm-vaaaa-aaaap-qb5kq-cai.raw.icp0.io/](https://htvbm-vaaaa-aaaap-qb5kq-cai.raw.icp0.io/).




