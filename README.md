# avail-appchain-demo
Demo of a simple app-specific chain built using Polygon Avail

## Use case
In this demo, we capture a very simple use case: matrix multiplication*.

Why? Because we wanted to take a simple non-commutative operation to show the power of ordering through Avail. 

*Ordinary multiplication means the values can get quite large and eventually overflow. Hence, after each block, we reduce the product matrix by reducing each element of the matrix modulo a small prime. 

## Setup
- Run Avail node

    For demo, a single node is enough. You can also try with multiple nodes where each app node also hosts an Avail node. Use [this link](https://github.com/maticnetwork/avail#run-node-for-development) for instructions. We use tag `v1.3.0`.

- Run Avail app client

    Run as many app clients you need. Ideally, for every application specific node, there should be an app client. Use [this link](https://github.com/maticnetwork/avail-light) to run app client. We use tag `v1.3.0`. Please note to change port numbers for each of these clients if running on the same system.

- Run application node

    Run the app node. Use `cargo run --bin app -- {port}` to run an instance, where `{port}` is the http server port of the corresponding Avail app client. 
    The node fetches app data from every block (based on appID). The app data contains a sequence of matrices. It computes the product and stores it modulo a small prime.

    Result: All the app nodes synced to the same height would have the same (reduced) product matrix. In a generic app chain sense, all synced app nodes would have the same post state. 

- Run transaction submitter script

    Run the txn submitter script. Use `cargo run --bin submit`. It assumes avail dev node is running locally on `9944`. It also assumes appID as `1`. It uses threads to submit one matrix per thread, generated randomly, each element being within a small prime. 

