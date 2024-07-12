# Platform Terminal User Interface (testnet only)

The Platform Terminal User Interface (TUI) is a command-line tool for Dash Evolution designed to run entirely in text-based terminals. It uses keyboard navigation and text commands to interact with the [Dash Platform](https://github.com/dashpay/platform) and with Evolution upgrade of the Dash network in general. TUI efficiently manages a wide range of tasks, such as creating and managing wallets, data contracts, registering identities, as well as performing more dynamic network actions like contract updates and identity transactions.

The primary purpose of TUI is to test the Platform's robustness and stability. One can test a simple transfer operation or manage a complex contract lifecycle. TUI can automatically submit the corresponding state transitions to the Platform at a specified rate per second for a given duration. To achieve efficient testing, TUI features structured declaration of testing Strategies. Strategies allow advanced users to customize and automate complex blockchain-related actions. Strategies facilitate running pre-defined tests consistently and efficiently by allowing to submit them to the Dash network repeatedly and regularly for each new development version.

The TUI also serves as a general-purpose tool for performing basic operations on the Dash Evolution network. These include handling of individual wallets, identities, contracts, and two-way transfers of native units between the Platform and the Core chains. TUI provides features for complex testing by advanced users, but at the same time its user interface is rather interactive and intuitive to use. Submitting individual commands with parameters to a command line prompt is simply prohibitive for some people. Thus TUI makes the interaction with the Dash technology more illustrative and accessible for more potential users.  

The Dash Platform Terminal User Interface (TUI) is a Rust-based user interface for interacting with Dash Platform in the terminal. Its purpose is to enable users to perform all actions permitted by Dash Platform, which, broadly speaking, means broadcasting state transitions and querying the network.

The TUI can connect to any instance of a Dash Platform network, including the testnet, devnets, local networks, and soon, the mainnet. However, for now this readme will only cover connecting to a testnet, and steps to connect to a local network will be added soon.

# Installation

TUI can be configured in three ways depending on the kind of network against which it is set up. The different types of setup and their dependencies are the following:​​​​

*   **Mainnet:** UNAVAILABLE (launch on July 29th, 2024). Similar to the following Testnet setup with the fully synchronized Dash Core node connected to mainnet rather than testnet.
    
*   **Testnet:** TUI runs against Dash testnet over the Internet. This simulates the mainnet behaviour in many ways. One needs to install Dash Core node configured for testnet. TUI is then configured according to the node.
    
*   **Devnet** TUI runs against local (development) network only. This is the most flexible, potentially the fastest way to try TUI. It is also the most complex set up that requires running one or more Dash Evo nodes to simulate the Dash Evolution network. This typically means use of other Dash-specific tool '[dashmate](%5Bhttps://%5D(https://github.com/dashpay/platform/tree/v1.0-dev/packages/dashmate))' in order to get Dockerized instances of Evo nodes with all necessary dependencies (Node.JS) and configuration.
    

## Testnet Installation

### Install Dependencies

Several packages are required to use the Dash SDK. Install them by running: Protobuf, Rust 

\# Install required packages
sudo apt-get install clang cmake gcc unzip
# Install recent protobuf version
wget https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-linux-x86\_64.zip
sudo unzip protoc-\*-linux-x86\_64.zip -d /usr/local  

Rustup is the [recommended tool](https://www.rust-lang.org/tools/install) to install Rust and keep it updated. To download Rustup and install Rust, run the following in your terminal, then follow the on-screen instructions:

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

After installation, ensure that your system’s environment variables are updated. Restart your terminal or run:

. "$HOME/.cargo/env"

Check if Rust is installed correctly by running:

rustc --version

You should see the installed version of Rust.

### Install Dash Core

Currently, the SDK is dependent on a Dash Core full node to support proof verification and provide wallet access. Follow the instructions below to install Dash Core and run it on Testnet. **Note:** it is possible, although not recommended, to retrieve data from Dash Platform without proof

*   [Dash Core installation instructions](https://docs.dash.org/en/stable/docs/user/wallets/dashcore/installation.html#dashcore-installation "Dash latest")
    
*   [Running Dash Core on Testnet](https://docs.dash.org/en/stable/docs/user/wallets/dashcore/advanced.html#dashcore-testnet "Dash latest")
    

and wait for the fully synchronized state of the node, which may take few hours.  

Locate the `dash.conf` file by right-clicking the Dash Core icon and selecting `Open Wallet Configuration File`. Configure it as shown below (replace `<username>` and `<password>` with values of your choice):

testnet=1
server=1
listen=1
rpcallowip=127.0.0.1
rpcuser=<user>
rpcpassword=<password>

Restart Dash Core to apply the changes.

> 🚧 **Using Dash Platform without Dash Core**
> 
> The Rust SDK requests proofs for all data retrieved from Platform. This makes it the recommended (most secure) option, but also is why a Dash Core full node is currently required.
> 
> The [JavaScript SDK](https://docs.dash.org/projects/platform/en/latest/docs/tutorials/introduction.html) provides access to Dash Platform without requiring a full node; however, it **_does not support Dash Platform’s proofs_**. The Rust DAPI client can also perform read operations without a full node if proofs are not requested. See the [DAPI client example](https://docs.dash.org/projects/platform/en/latest/docs/sdk-rs/quick-start.html#dapi-client-example) below for details.  

  

  

  

1.  First, you need to run a Dash Core testnet node in order to connect to the testnet. Download the latest version of Dash Core [here](https://www.dash.org/downloads/#desktop). Run Dash Core and then configure the \`dash.conf\` file as follows, replacing \`\*\*\*\` with a username and password of your choosing (find the \`dash.conf\` file by right-clicking the Dash Core icon and selecting \`Open Wallet Configuration File\`):
    
        server=1
        listen=1
        rpcallowip=127.0.0.1
        rpcuser=***
        rpcpassword=***
        testnet=1
        
    
    Restart Dash Core for the changes to take effect.
    
2.  Next, clone the [TUI repo](https://github.com/dashpay/rs-platform-explorer):
    
        git clone https://github.com/dashpay/rs-platform-explorer.git
        
    
3.  Open the TUI repo in your terminal and install Rust
    
        cd rs-platform-explorer
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        
    
    After installing Rust, restart your terminal and navigate back to rs-platform-explorer.
    
4.  Add the WebAssembly target to Rust
    
        rustup target add wasm32-unknown-unknown
        
    
5.  Install build-essential tools, SSL development libraries, and Clang. On Ubuntu, use:
    
        sudo apt install -y build-essential libssl-dev pkg-config clang
        
    
    On other Unix-like systems, use the equivalent package management commands.
    
6.  Install wasm-bindgen-cli:
    
        cargo install wasm-bindgen-cli@0.2.85
        
    

1.  Install Protocol Buffers Compiler (protoc):
    1.  Download the appropriate protoc binary for your system:
        
            wget https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-linux-x86_64.zip
            
        
    2.  Install unzip if not already installed:
        
            sudo apt install unzip
            
        
    3.  Unzip and install \`protoc\`:
        
            sudo unzip protoc-*-linux-x86_64.zip -d /usr/local
            
        
2.  Install CMake:
    
        sudo apt update
        sudo apt install cmake
        
    
3.  Now, create a \`.env\` file in the highest level of the TUI directory, and copy the contents of \`.env.testnet\` into it
4.  Set the username and password in the \`.env\` file to the username and password in your Dash Core \`wallet.conf\` file and save
5.  Do \`cargo run\` to start the TUI
    
        cargo run
    

# Examples

Now that we are inside the TUI, we need to load a wallet and an identity.

*   Go to the Wallet screen and add a wallet by private key. You can generate a private key on [this website](https://passwordsgenerator.net/sha256-hash-generator/) by typing in a seed (it can be any random word or phrase), and then paste that key into the TUI. (take note: the key has to be in hexadecimal format!)
*   After entering the private key, copy the Wallet receive address and paste it into the [testnet faucet](https://faucet.testnet.networks.dash.org/) to get some Dash funds. Use promo code `platform` to get 50 DASH (normally it only dispenses around 2-5).
*   In the TUI, refresh the wallet balance until the funds appear.
*   Register an identity and fund it with 1 DASH. This is more than enough for the example test we’ll be running.
*   Refresh the identity balance until you see the funds appear.

* * *

Now, we’re ready to build and run a strategy. for further indication on how to do so, you can look [Paul Delucia's guide](https://www.dash.org/blog/strategy-tests-usage-guide/)

  

Now, we’re ready to build and run a strategy. We’re going to register 20 start\_contracts, 10 start\_identities, and broadcast 1 document per contract per second for 5 minutes, making it 20 tx/s for 5 minutes.

1.  Go back to the main screen and open the Strategies screen
2.  Create a new strategy and name it whatever you want
3.  First, we’ll add start\_identities. So go to the Start Identities screen and add 10 identities with 3 keys each and no transfer key. Then, set the balance of the identities to 0.2 DASH. This will be just enough to cover the entire strategy run. Go back to the Strategy screen.
4.  Next, we’ll add start\_contracts. Go to the Start Contracts screen and press \`x\`. Select a contract and create 20 variants. Go back to the Strategy screen.
5.  Now, go to the Operations screen, press \`x\`, select “1” for one document per contract (it defaults to the first document type of the contract alphabetically), select “Minimum” to insert the minimum amount of data required for each document, and select “No” in order to not populate fields that aren’t required. Go back to the Strategy screen.
6.  The Strategy is now ready to run. Press \`r\` to run the strategy. Select \`second\` mode to run 20 tx/s. Enter \`300\` to run for 5 minutes. If you select \`second\` mode, it doesn’t matter whether you choose to verify proofs or not – they aren’t verified in second mode either way. Confirm you would like to run the strategy.

Now, the strategy should begin initialization and then execution. You can check the \`explorer.log\` file to see the progress. You may also check the [testnet block explorer](http://platform-explorer.com/) to see your strategy in action, as well as the public [Grafana metrics](http://metrics.testnet.networks.dash.org/grafana/public-dashboards/5b1f9fc67dee4cad94a19b3dcbe1d24d) UI hosted by Dash Core Group, which shows metrics like average tx/s and mempool size. When the strategy is finished running, the execution results will be displayed in the TUI. Next, we’ll go into a bit more detail on the Strategy structure which defines strategy tests.

## Strategy Structure

Strategies are defined with the _Strategy_ structure ([GitHub](https://github.com/dashpay/platform/blob/b322c81cdeb98412bce154d6389c3fb156d2a7ba/packages/strategy-tests/src/lib.rs#L103)): 

    struct Strategy {
       start_identities,
       start_contracts,
       operations,
       identity_inserts,
    }

  

### start\_identities

This field defines the identities to insert into the state at the start of the strategy. They will be used to broadcast the rest of the state transitions. Users can specify the number of identities, the number of keys to add to each identity, whether or not to include a special key for _Credit Transfers_, and the starting balance of each identity.

### start\_contracts

This field defines the contracts to register at the start of the strategy. It also allows users to define contract updates to be executed on the initial contracts after a certain number of blocks or seconds (for now the updates are hardcoded to happen every 3 blocks or seconds). The TUI reads from the \`supporting-files/contract\` directory when giving users the option of contracts to register, so you can add contracts to that directory if you wish.

### operations

Operations are the state transitions to be executed for the duration of the strategy after the start\_identities and start\_contracts are registered. They can be any of the supported state transitions, which include _Document Inserts_, _Contract Inserts_, _Contract Updates_, _Identity Inserts_, _Identity Updates_, _Identity Top Ups_, _Credit Transfers_, and _Credit Withdrawals_. Users can specify the number of times to execute each state transition per block or second, the percent chance that the operation is executed per block or second, and other state transition-specific parameters; for example, for a Contract Create, they can select which contract to insert, and for a Document Insert, they can specify whether or not to populate not-required fields. Users can create any combination of operations to execute per block or second at their desired frequency. This is where the bulk of the strategy test execution happens.

### identity\_inserts

This field does the same thing as the operations field, just for Identity Inserts. It will probably be merged into the operations field at some point.

## Strategy Execution

Once the user has defined or loaded their strategy, wallet, and identity into the TUI, they can run the strategy test after specifying the execution mode: block mode or second mode. Block mode is designed to prepare and broadcast the state transitions on a per-block basis, while second mode is designed to broadcast the state transitions on a per-second basis. Users are then able to specify the number of blocks or seconds to execute the strategy, and whether or not they would like to verify the proofs returned from the _StateTransitionExecutionResults_ (proof verification only applies to block mode).

Once the strategy execution is confirmed, two things will happen before any state transitions are submitted. First, the nonces of the TUI’s Loaded Identity for all the contracts the strategy interacts with are fetched. Then, asset lock proofs need to be obtained for all the _Identity Inserts_ and _Top Ups_. Normally, the asset lock proof creation rate is about 2 per second, so it may take some time if you’re doing a lot of these transitions. You can check the \`explorer.log\` file to see the real-time progress.

After these initialization steps, the start\_identities are inserted. The strategy will broadcast the state transitions and then wait for nodes to respond with _StateTransitionExecutionResults_, ensuring that the identities have been inserted into the state (unless there’s an error) before moving on. Then, the start\_contracts are inserted, using the start\_identities as owners. We again broadcast the state transitions and then wait for the nodes to respond with _StateTransitionExecutionResults_ to confirm all the state transitions have been processed before moving on. Finally, the execution of the operations begins and runs for the course of the strategy. In time mode, the state transitions from here on out are only broadcast and there is no waiting for results. In block mode, we do wait for the results, and optionally verify the proofs as well.

The progress of the strategy can be followed in the \`explorer.log\` file of the TUI.

Once the strategy is completed, execution results will be displayed.

## Importing Strategies

One notable feature of the TUI is that users can import and export strategies. When you export a strategy, a binary file of the _Strategy_ structure is created in \`rs-platform-explorer/supporting-files/strategy-exports\`. This binary file can be added to a Github repository and later imported into the TUI via the raw Github link. Dash Core Group maintains a [repository of strategies](https://github.com/dashpay/platform-strategy-tests) along with versioned execution results against the Platform testnet. Of course, this means users are free to import these strategies and run them themselves. This also makes it easy for users to create their own repositories in a similar fashion.

## Nonce Errors

One of the biggest hurdles in creating an effective time-based strategy test is getting around nonce errors. Basically, what one needs to know is that identities are limited to 24 transitions per block, or in the case of Document Inserts, 24 documents per contract per block. Since we’re usually submitting state transitions at a rate that is faster than they are being included in blocks, and the nonces need to be set in the state transitions before we send them, the nonces can get too far ahead of what’s actually in the chain state, and so we get errors about them being “too far in the future”. An example of how to get around nonce errors is as follows: if a user wants to do 80 Document Inserts per second, they should do something like register 80 contracts at the start, 30 identities, and then do 1 Document Insert per contract per second. This specific combination consistently produces 99% state transition success rate for a strategy executing at 80 Document Inserts per second for 10 minutes, as can be seen in the results in the [repository of strategies](https://github.com/dashpay/platform-strategy-tests).