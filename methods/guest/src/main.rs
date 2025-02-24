#![no_main]
#![no_std]

extern crate alloc;

use contract::Counter;
use sdk::guest::execute;
use sdk::guest::GuestEnv;
use sdk::guest::Risc0Env;

risc0_zkvm::guest::entry!(main);

fn main() {
    //
    // Usually you don't need to update this file.
    // Except to specify the name of your contract type (here = Counter)
    //

    let env = Risc0Env {};
    let input = env.read();
    let (_, output) = execute::<Counter>(&input);
    env.commit(&output);
}
