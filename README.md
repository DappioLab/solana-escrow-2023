# Solana Escrow 2023 Ver

This is a rewrite of the legendary [blog post](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/#edits-and-acknowledgements)/[repo](https://github.com/paul-schaaf/solana-escrow)

Added some new stuff include all Program accounts are PDA, uses ATA for Token accounts, separate the logic and checking.

# How to test

- Start a local-validator in the background

```bash
solana-test-validator -r -q
```

- build the program and deploy

```bash
cargo build-sbf
solana airdrop 2
solana program deploy ./target/deploy/solana_escrow_plus.so
```

- run test script

```bash
cd tests
yarn
yarn test
```
