# Substrate Runtime Multisig Wallet

A development version of multi-signature wallet on Substrate, started at Blockchain Hackathon 21.09.2019

# How to use it:

 * Run `git clone https://github.com/f3joule/f3joule-multisig-subwallet/tree/multisig-wallet`.
 * Cd to a root of the project.
 * Run `curl https://getsubstrate.io -sSf | bash -s -- --fast`
    * This installs external dependencies needed for substrate. [Take a look at the script](https://getsubstrate.io).
    * The `--fast` command allows us to skip the `cargo install` steps for `substrate` and `subkey`, which is not needed for runtime development.

* Go into the `multisig-subwallet` folder and run:
    * `./scripts/build-runtime.sh`
    * `cargo build`
    * `cargo run -- --dev`
    * This should start your node, and you should see blocks being created

* Go into the `multisig-subwallet-ui` folder and run:
    * `yarn install`
    * `yarn dev`
    * This should start a web server on `localhost:8000` where you can interact with your node

* Interact with your node and hack away!
