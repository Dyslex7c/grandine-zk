use std::{io::Read, path::Path, str::FromStr, time::Instant};

use anyhow::Result;
use backend::{Vm, VmBackend as _};
use bls as _;
use clap::{Parser, Subcommand};
use ssz::{SszHash as _, SszRead as _, H256};
use transition_functions::combined::untrusted_state_transition as state_transition;
use types::{
    combined::{BeaconState, SignedBeaconBlock},
    config::Config,
    preset::Mainnet,
    traits::BeaconState as _,
};

use crate::backend::{ProofTrait, ReportTrait};

mod backend;

#[derive(Clone, Debug)]
struct Test {
    name: &'static str,

    block: &'static str,
    state: &'static str,

    expected_slot: u64,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    test: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Execute,
    Prove,
}

fn main() -> Result<()> {
    // let file = std::fs::File::open(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("e577e646-270b-409d-a76c-31aa81df8f9b.bincode"))?;
    // let mut buf = Vec::new();
    // file.read_to_end(&mut buf)?;
    // let receipt: risc0_zkvm::Receipt = bincode::deserialize(&buf)?;
    // println!("cycles: {}", receipt.);

    let tests = [
        Test {
            name: "pectra-devnet-6 first block",

            block: "../data/pectra-devnet-6/beacon_block_slot_00000001_root_0x58602aaed9e485527f8fdaafef2000398a722d091eb6619c28c669acc69547ef.ssz",
            state: "../data/pectra-devnet-6/genesis.ssz",

            expected_slot: 1,
        },
        Test {
            name: "pectra-devnet-6 with epoch transition",

            block: "../data/pectra-devnet-6/beacon_block_slot_00021568_root_0xb28a634b89c669141990ed5deceb1ea4777869a64cb8eaccb6cb9f4796c5110d.ssz",
            state: "../data/pectra-devnet-6/beacon_state_slot_00021567_root_0xd51b605669c3e1ec96d83b6ab191d921f276d363621009fa6fd4a171a6bbf943.ssz",

            expected_slot: 21568,
        },
        Test {
            name: "pectra-devnet-6 without epoch transition",

            block: "../data/pectra-devnet-6/beacon_block_slot_00021569_root_0x91008e253d2dafd1c9cd6a8ccae68a3d3010a85697ba588ef0be3dcb9b93332d.ssz",
            state: "../data/pectra-devnet-6/beacon_state_slot_00021568_root_0xb28a634b89c669141990ed5deceb1ea4777869a64cb8eaccb6cb9f4796c5110d.ssz",

            expected_slot: 21569,
        },
        Test {
            name: "mainnet without epoch transition",

            block: "../data/mainnet/beacon_block_slot_11893759_root_0x3a74cd235bf22d0d637b41b320f9162c6a7c81639b3fff28d1bceb1627fe82fb.ssz",
            state: "../data/mainnet/beacon_state_slot_11893758_root_0x6ae5cfd675459d878fc43a4205967660abc21e8e399195da5013af6b0547420b.ssz",

            expected_slot: 11893759,
        }
    ];

    let args = Args::parse();

    let selected_test = tests
        .iter()
        .find(|i| i.name.contains(&args.test))
        .expect("No matching test");

    println!("Running test \"{}\"", selected_test.name);

    let config = Config::pectra_devnet_6();

    let block_ssz = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(selected_test.block))?;

    let state_ssz = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(selected_test.state))?;

    let expected_root = {
        let block = SignedBeaconBlock::<Mainnet>::from_ssz(&config, block_ssz.clone())?;
        let mut state = BeaconState::<Mainnet>::from_ssz(&config, state_ssz.clone())?;

        state_transition(&config, &mut state, &block)?;
        state.hash_tree_root()
    };

    match args.command {
        Command::Execute => {
            let started_at = Instant::now();
            let vm = Vm::new()?;
            let (output_bytes, report) = vm.execute(state_ssz, block_ssz)?;
            let state = BeaconState::<Mainnet>::from_ssz(&config, output_bytes)?;

            println!("elapsed: {:?}", started_at.elapsed());
            println!("cycles: {}", report.cycles());

            println!("state slot after state transition: {}", state.slot());
            println!(
                "state root after state transition: {:?}",
                state.hash_tree_root()
            );
            assert_eq!(state.slot(), selected_test.expected_slot);
            assert_eq!(state.hash_tree_root(), expected_root);
        }
        Command::Prove => {
            let started_at = Instant::now();
            let vm = Vm::new()?;
            let (output_bytes, proof) = vm.prove(state_ssz, block_ssz)?;
            let state = BeaconState::<Mainnet>::from_ssz(&config, output_bytes)?;
            println!("elapsed: {:?}", started_at.elapsed());
            // println!("cycles: {}", proof.cycles());
            println!("state slot after state transition: {}", state.slot());
            println!(
                "state root after state transition: {:?}",
                state.hash_tree_root()
            );

            proof.save(Path::new(env!("CARGO_MANIFEST_DIR")).join("proof.bin"))?;

            assert_eq!(proof.verify(), true);
            assert_eq!(state.slot(), selected_test.expected_slot);
            assert_eq!(state.hash_tree_root(), expected_root);
        }
    }

    Ok(())
}
