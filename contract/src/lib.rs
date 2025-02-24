use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{Digestable, HyleContract, RunResult};

impl HyleContract for Counter {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<CounterAction>(contract_input)?;

        // Execute the contract logic
        match action {
            CounterAction::Increment => self.value += 1,
        }

        // program_output might be used to give feedback to the user
        let program_output = format!("new value: {}", self.value);
        Ok((program_output, ctx, vec![]))
    }
}

/// The action represents the different operations that can be done on the contract
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum CounterAction {
    Increment,
}

/// The state of the contract, in this example it is fully serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct Counter {
    pub value: u32,
}

/// Utils function for the host
impl Counter {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

/// Utils function for the host
impl CounterAction {
    pub fn as_blob(&self, contract_name: &str) -> sdk::Blob {
        sdk::Blob {
            contract_name: contract_name.into(),
            data: sdk::BlobData(borsh::to_vec(self).expect("failed to encode BlobData")),
        }
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for Counter {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode Balances"))
    }
}
impl From<sdk::StateDigest> for Counter {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
