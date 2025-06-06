use std::path::Path;

use anyhow::Result;

pub trait ReportTrait {
    fn cycles(&self) -> u64;
}

pub trait ProofTrait {
    fn verify(&self) -> bool;

    fn save(&self, path: impl AsRef<Path>) -> Result<()>;
}

pub trait VmBackend: Sized {
    type Report: ReportTrait;
    type Proof: ProofTrait;

    fn new() -> Result<Self>;

    fn execute(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Report)>;

    fn prove(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Proof)>;
}

#[cfg(feature = "risc0")]
mod risc0 {
    use std::{fs::File, io::BufWriter};

    use borsh::BorshSerialize;
    use super::{VmBackend, ReportTrait, ProofTrait};
    use anyhow::Result;
    use risc0_zkvm::{default_prover, ExecutorEnv, Receipt, SessionStats, serde::to_vec};
    use zkvm_guest_risc0::{RISC0_GRANDINE_STATE_TRANSITION_ELF, RISC0_GRANDINE_STATE_TRANSITION_ID};

    pub struct Vm;

    pub struct Report(SessionStats); 

    impl ReportTrait for Report {
        fn cycles(&self) -> u64 {
            self.0.total_cycles
        }
    }

    pub struct Proof(Receipt);

    impl ProofTrait for Proof {
        fn verify(&self) -> bool {
            self.0.verify(RISC0_GRANDINE_STATE_TRANSITION_ID).is_ok()
        }
    
        fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
            let mut writer = BufWriter::new(File::create(path)?);
            self.0.serialize(&mut writer)?;

            Ok(())
        }
    }

    impl VmBackend for Vm {
        type Report = Report;
        
        type Proof = Proof;
    
        fn new() -> Result<Self> {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
                .init();
            Ok(Self)
        }
    
        fn execute(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Report)> {
            let prover = default_prover();

            let env = ExecutorEnv::builder()
                .write(&state_ssz.len())?
                .write(&block_ssz.len())?
                .write_slice(&state_ssz)
                .write_slice(&block_ssz)
                .build()?;

            let elf = RISC0_GRANDINE_STATE_TRANSITION_ELF;

            let prove_info = prover.prove(env, elf)?;
            let receipt = prove_info.receipt;
            
            Ok((receipt.journal.bytes, Report(prove_info.stats)))
        }

        fn prove(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Proof)> {
            let prover = default_prover();

            let env = ExecutorEnv::builder()
                .write(&state_ssz.len())?
                .write(&block_ssz.len())?
                .write_slice(&state_ssz)
                .write_slice(&block_ssz)
                .build()?;

            let elf = RISC0_GRANDINE_STATE_TRANSITION_ELF;

            let prove_info = prover.prove(env, elf)?;
            let receipt = prove_info.receipt;
            
            Ok((receipt.journal.bytes.clone(), Proof(receipt)))
        }
    }
}

#[cfg(feature = "risc0")]
pub use risc0::*;

#[cfg(feature = "sp1")]
mod sp1 {
    use super::{VmBackend, ReportTrait, ProofTrait};
    use sp1_sdk::{include_elf, ProverClient, SP1Stdin, ExecutionReport, SP1ProofWithPublicValues, EnvProver, SP1VerifyingKey};
    use anyhow::Result;
    use std::path::Path;

    const STATE_TRANSITION_ELF: &[u8] = include_elf!("zkvm_guest_sp1");

    pub struct Report(ExecutionReport);

    impl ReportTrait for Report {
        fn cycles(&self) -> u64 {
            self.0.total_instruction_count()
        }
    }

    pub struct Proof(EnvProver, SP1VerifyingKey, SP1ProofWithPublicValues);

    impl ProofTrait for Proof {
        fn verify(&self) -> bool {
            self.0.verify(&self.2, &self.1).is_ok()
        }

        fn save(&self, path: impl AsRef<Path>) -> Result<()> {
            self.2.save(path)
        }
    }

    pub struct Vm;

    impl VmBackend for Vm {
        type Report = Report;
        
        type Proof = Proof;
    
        fn new() -> Result<Self> {
            sp1_sdk::utils::setup_logger();

            Ok(Vm)
        }
    
        fn execute(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Report)> {
            let client = ProverClient::from_env();
            let mut stdin = SP1Stdin::new();

            stdin.write_slice(&state_ssz);
            stdin.write_slice(&block_ssz);

            let (output, report) = client.execute(STATE_TRANSITION_ELF, &stdin).run()?;
        
            Ok((output.as_slice().to_vec(), Report(report)))
        }
        
        fn prove(&self, state_ssz: Vec<u8>, block_ssz: Vec<u8>) -> Result<(Vec<u8>, Self::Proof)> {
            let client = ProverClient::from_env();

            let (pk, vk) = client.setup(STATE_TRANSITION_ELF);

            let mut stdin = SP1Stdin::new();

            stdin.write_slice(&state_ssz);
            stdin.write_slice(&block_ssz);

            let proof = client.prove(&pk, &stdin).run()?;

            Ok((proof.public_values.as_slice().to_vec(), Proof(client, vk, proof)))
        }
    }
}

#[cfg(feature = "sp1")]
pub use sp1::*;
