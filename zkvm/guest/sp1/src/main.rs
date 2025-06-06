// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use anyhow::Result;
use ssz::{SszRead as _, SszWrite as _};
use transition_functions::combined::untrusted_state_transition as state_transition;
use types::{
    combined::{BeaconState, SignedBeaconBlock},
    config::Config,
    preset::{Mainnet, Preset},
};

fn read_block_and_state<P: Preset>(
    config: &Config,
) -> Result<(SignedBeaconBlock<P>, BeaconState<P>)> {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let state_ssz = sp1_zkvm::io::read_vec();
    let block_ssz = sp1_zkvm::io::read_vec();

    let block = SignedBeaconBlock::<P>::from_ssz(config, &block_ssz)?;
    let state = BeaconState::<P>::from_ssz(config, &state_ssz)?;

    Ok((block, state))
}

pub fn main() {
    //let config = Config::pectra_devnet_4();
    let config = Config::pectra_devnet_6();

    println!("loading block and state...");

    let (block, mut state) = read_block_and_state::<Mainnet>(&config).unwrap();

    println!("loaded block and state");

    println!("performing state transition...");

    state_transition(&config, &mut state, &block).unwrap();

    println!("performed state transition");


    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&state.to_ssz().unwrap());

    println!("committed output");
}
