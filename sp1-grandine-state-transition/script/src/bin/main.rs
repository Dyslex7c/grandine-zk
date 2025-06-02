//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_sol_types::SolType;
use clap::Parser;
use fibonacci_lib::PublicValuesStruct;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

use anyhow::Result;
use ssz::{SszHash as _, SszRead as _, SszWrite as _};
use types::{
    combined::BeaconState, config::Config,
    deneb::containers::SignedBeaconBlock as DenebSignedBeaconBlock, phase0::primitives::H256,
    preset::Mainnet, traits::BeaconState as _,
};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, default_value = "20")]
    n: u32,
}

fn main() -> Result<()> {
    let config = Config::pectra_devnet_6();

    println!("loading data...");

//  without epoch transition
/*    let block_ssz = std::fs::read(
        "./data/block_224321_0x25f01d85065e405a0097f7ba60092d55728ca2815ac7d8cd9b8244f6eaeedd22.ssz",
    )?;

    let electra_block = ElectraSignedBeaconBlock::<Mainnet>::from_ssz(&config, &block_ssz)?;

    let state_ssz = std::fs::read(
        "./data/state_224320_0xb6be0a7bc632809e85c9176db3f195418c29c1cfeab559be094de61339ba97ee.ssz",
    )?;
*/

// with epoch transition
/* let block_ssz = std::fs::read(
        "./data/block_224320_0xb6be0a7bc632809e85c9176db3f195418c29c1cfeab559be094de61339ba97ee.ssz",
    )?;

    let electra_block = ElectraSignedBeaconBlock::<Mainnet>::from_ssz(&config, &block_ssz)?;

    let state_ssz = std::fs::read(
        "./data/state_224319_0x7ece9f54e1112ba2b006ce90903015527e8bd4ca3c3abfff991c281b7dd19c4c.ssz",
    )?;
*/

    let block_ssz = std::fs::read(
        "../data/pectra-devnet-6/beacon_block_slot_00000001_root_0x58602aaed9e485527f8fdaafef2000398a722d091eb6619c28c669acc69547ef.ssz",
    )?;
    println!("1");

    // let electra_block = DenebSignedBeaconBlock::<Mainnet>::from_ssz(&config, &block_ssz)?;

    let state_ssz = std::fs::read(
        "../data/pectra-devnet-6/genesis.ssz",
    )?;
    println!("2");

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    // stdin.write(&args.n);

    // println!("n: {}", args.n);

    stdin.write_slice(&block_ssz);
    stdin.write_slice(&state_ssz);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(FIBONACCI_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let state_ssz = output.as_slice();

        let state = BeaconState::<Mainnet>::from_ssz(&config, state_ssz)?;

        let state_root = state.hash_tree_root();

        println!("state slot after state transition: {}", state.slot());
        println!("state root after state transition: {state_root:?}");

        // let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        // let PublicValuesStruct { n, a, b } = decoded;
        // println!("n: {}", n);
        // println!("a: {}", a);
        // println!("b: {}", b);

        // let (expected_a, expected_b) = fibonacci_lib::fibonacci(n);
        // assert_eq!(a, expected_a);
        // assert_eq!(b, expected_b);
        // println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(FIBONACCI_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }

    Ok(())
}
