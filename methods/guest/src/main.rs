#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use contract::Counter;
use sdk::guest::execute;
use sdk::guest::GuestEnv;
use sdk::guest::Risc0Env;
use sdk::Calldata;

risc0_zkvm::guest::entry!(main);

fn main() {
    //
    // Usually you don't need to update this file.
    // Except to specify the name of your contract type (here = Counter)
    //

    let env = Risc0Env {};
    let (commitment_metadata, calldata): (Vec<u8>, Calldata) = env.read();

    let output = execute::<Counter>(&commitment_metadata, &calldata);
    env.commit(&output);
}
