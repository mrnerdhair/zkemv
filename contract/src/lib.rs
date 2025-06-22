use std::collections::BTreeMap;

use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use sdk::FullStateRevert;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use rsa::traits::PublicKeyParts;

impl sdk::ZkContract for ZkEmv {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::Calldata) -> sdk::RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<ZkEmvAction>(contract_input)?;

        let icc_key_hash = hex::decode(
            contract_input.identity.0.split('@').nth(0).ok_or("identity extraction error")?,
        ).or(Err("identity decoding error".to_string()))?
        .try_into().or(Err("identity size error".to_string()))?;

        // Execute the contract logic
        let output = match action {
            ZkEmvAction::RegisterIdentity => {
                if self.identities.insert(icc_key_hash, 1).is_some() { return Err("identity already exists".to_string()); }
                format!("Registered identity {}", hex::encode(icc_key_hash))
            },
            ZkEmvAction::VerifyIdentity => {
                let card_things = CardThings::from_bytes(&contract_input.private_input);
                let nonce = self.identities.get_mut(&icc_key_hash).ok_or("identity not found".to_string())?;
                if icc_key_hash != card_things.icc_key_hash() { return Err("icc key hash mismatch".to_string()) }
                card_things.verify_card_things(*nonce).or(Err("verification failed".to_string()))?;
                *nonce = nonce.checked_add(1).ok_or("nonce overflow".to_string())?;
                format!("Verified identity {}; nonce is now {}", hex::encode(icc_key_hash), *nonce)
            },
        };

        Ok((output.as_bytes().to_vec(), ctx, vec![]))
    }

    /// Commit the state of the contract
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(borsh::to_vec(self).expect("Failed to encode state"))
    }
}

/// The action represents the different operations that can be done on the contract
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ZkEmvAction {
    RegisterIdentity,
    VerifyIdentity,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CardThings {
    pub icc_pk_modulus: Vec<u8>,
    pub icc_pk_exponent: Vec<u8>,
    pub arqc_sig_raw: Vec<u8>,
    pub arqc_sig_hash_contents: Vec<u8>,
}

impl CardThings {
    pub fn icc_key_hash(&self) -> [u8; 32] {
        sha2::Sha256::digest(&[
            u32::try_from(self.icc_pk_modulus.len()).unwrap().to_be_bytes().as_slice(),
            self.icc_pk_modulus.as_slice(),
            u32::try_from(self.icc_pk_exponent.len()).unwrap().to_be_bytes().as_slice(),
            self.icc_pk_exponent.as_slice(),
        ].concat()).try_into().unwrap()
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        borsh::from_slice(&buf)
            .map_err(|_| "Could not decode card things".to_string())
            .unwrap()
    }

    pub fn verify_card_things(&self, nonce: u32) -> Result<(), ()> {
        let icc_key = rsa::RsaPublicKey::new(
            rsa::BigUint::from_bytes_be(&self.icc_pk_modulus),
            rsa::BigUint::from_bytes_be(&self.icc_pk_exponent),
        ).or(Err(()))?;

        if icc_key.n().bits() != self.arqc_sig_raw.len() * 8 { return Err(()); }
        let arqc_sig: rsa::BigUint = rsa::hazmat::rsa_encrypt(&icc_key, &rsa::BigUint::from_bytes_be(&self.arqc_sig_raw)).or(Err(()))?;
        let arqc_sig = arqc_sig.to_bytes_be();

        if icc_key.n().bits() != arqc_sig.len() * 8 { return Err(()); }
        if *arqc_sig.last().unwrap() != 0xbc { return Err(()); }
        if arqc_sig[0] != 0x6a { return Err(()); }
        if arqc_sig[1] != 0x05 { return Err(()); }

        let arqc_sig_hash: Vec<u8> = sha1::Sha1::digest(&self.arqc_sig_hash_contents).to_vec();

        let arqc_cert_hash_expected = &arqc_sig[(arqc_sig.len() - 21)..][0..20];

        if &arqc_sig_hash != arqc_cert_hash_expected { return Err(()); }

        if u32::from_be_bytes(self.arqc_sig_hash_contents[(self.arqc_sig_hash_contents.len() - 4)..].try_into().unwrap()) != nonce { return Err(()); }

        Ok(())
    }
}

/// The state of the contract, in this example it is fully serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ZkEmv {
    identities: BTreeMap<[u8; 32], u32>,
}

impl FullStateRevert for ZkEmv {}

/// Utils function for the host
impl ZkEmv {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

/// Utils function for the host
impl ZkEmvAction {
    pub fn as_blob(&self, contract_name: &str) -> sdk::Blob {
        sdk::Blob {
            contract_name: contract_name.into(),
            data: sdk::BlobData(borsh::to_vec(self).expect("failed to encode BlobData")),
        }
    }
}

impl From<sdk::StateCommitment> for ZkEmv {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode zkevm state".to_string())
            .unwrap()
    }
}

impl ZkEmv {
    pub fn get_nonce(&self, icc_key_hash: [u8; 32]) -> Option<u32> {
        self.identities.get(&icc_key_hash).copied()
    }
}
