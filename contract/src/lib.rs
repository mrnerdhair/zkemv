use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

impl sdk::ZkContract for Counter {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::Calldata) -> sdk::RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<CounterAction>(contract_input)?;

        // Execute the contract logic
        match action {
            CounterAction::Increment => self.value += 1,
        }

        // program_output might be used to give feedback to the user
        let program_output = format!("new value: {}", self.value);
        Ok((program_output, ctx, vec![]))
    }

    /// Commit the state of the contract
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(borsh::to_vec(self).expect("Failed to encode Balances"))
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

impl From<sdk::StateCommitment> for Counter {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
