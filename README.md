# schnorr_canister

Mock canister to create schnorr signatures on the Internet Computer.

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

## ToDos

- [ ] Implement key derivation
- [ ] Require cycles
- [ ] Try to get rid of `getrandom` dependency or provide proper implementation of custom function
- [ ] Try to init key in `init` function. Maybe with timer.

## Feedback

- `--no-frontend` flag is still not working
- CDK initialized with outdated versions

