//! src/helpers/artifact.rs
//! A small helper module to read a Forge artifact JSON and extract the creation code.

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use std::fs;

/// This mirrors the JSON structure: "bytecode": { "object": "...hex..." }
#[derive(Debug, Deserialize)]
struct BytecodeObject {
    object: String,
}

/// The top-level artifact with a "bytecode" field.
#[derive(Debug, Deserialize)]
struct MyContractArtifact {
    bytecode: BytecodeObject,
}

/// Reads the given JSON file (a Forge artifact) and returns the raw creation code bytes.
///
/// * `filename`: path to the artifact JSON (e.g. `../onchain/out/Push3Interpreter.sol/Push3Interpreter.json`)
pub fn get_creation_code(filename: &str) -> Result<Vec<u8>> {
    // 1) Read the JSON file
    let creation_json = fs::read_to_string(filename)
        .map_err(|e| anyhow!("Failed to read JSON file {}: {}", filename, e))?;
    
    // 2) Parse
    let contract_artifact: MyContractArtifact = serde_json::from_str(&creation_json)
        .map_err(|e| anyhow!("Failed to parse JSON artifact: {}", e))?;

    // 3) Extract the hex code
    let raw_hex = &contract_artifact.bytecode.object;
    let stripped_hex = raw_hex.trim_start_matches("0x");

    // 4) Convert from hex => bytes
    match hex::decode(stripped_hex) {
        Ok(bytes) => Ok(bytes),
        Err(e) => bail!("Invalid hex for creation code: {e}"),
    }
}
