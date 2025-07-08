//! src/evm_integration/revm_runner.rs
//! 
//! This module provides a thin wrapper around REVM to deploy a Push3Interpreter
//! contract, then call its `runInterpreter(...)` function with flexible inputs/outputs,
//! using the same style (Context::builder(), .modify_tx_chained, etc.) you had in your main.

use anyhow::{anyhow, bail, Result};
use ethers::abi::{encode, decode, Token, ParamType};
use ethers::types::U256;
use ethers::utils;
use database::CacheDB;
use revm::{
    context::Context,
    context::{BlockEnv, TxEnv, CfgEnv},
    context_interface::result::{ExecutionResult, Output},
    database_interface::EmptyDB,
    handler::EthHandler,
    primitives::{Bytes, TxKind}, // 4 generics
    EvmCommit,
    MainEvm,
};

// We import your AST definitions so we can call `ast.to_bytecode()`.
use crate::compiler::ast::{UntypedAst, Push3Ast};

// If you have a descriptor helper (like make_sublist_descriptor), bring it in:
use crate::compiler::push3_describtor::make_sublist_descriptor;

/// The input parameters for `runInterpreter(...)`: five fields (code, codeStack, execStack, intStack, boolStack).
pub struct Push3InterpreterInputs {
    pub code: Vec<u8>,
    pub init_code_stack: Vec<U256>,
    pub init_exec_stack: Vec<U256>,
    pub init_int_stack: Vec<i128>,
    pub init_bool_stack: Vec<bool>,
}

/// The outputs from `runInterpreter(...)`: four arrays for code/exec/int/bool stacks.
pub struct Push3InterpreterOutputs {
    pub final_code_stack: Vec<U256>,
    pub final_exec_stack: Vec<U256>,
    pub final_int_stack: Vec<i128>,
    pub final_bool_stack: Vec<bool>,
}

/// A thin wrapper around REVM, parameterized by the 4 generics (DB, BLOCK, TX, CFG).
/// - We store the ephemeral EVM instance,
/// - We store the deployed address of your `Push3Interpreter`,
/// - We provide `run_interpreter` and `run_ast`.
pub struct EvmRunner {
    /// The 4-parameter MainEvm:
    ///   DB = CacheDB<EmptyDB>,
    ///   BLOCK = BlockEnv,
    ///   TX = TxEnv,
    ///   CFG = CfgEnv
    pub evm: MainEvm<CacheDB<EmptyDB>, BlockEnv, TxEnv, CfgEnv>,

    /// The address where Push3Interpreter was deployed.
    pub interpreter_addr: revm::primitives::Address,
}

impl EvmRunner {
    /// Deploy a new ephemeral EVM with a "Create" transaction for the given `creation_code`.
    ///
    /// This matches your older style of:
    ///  `MainEvm::new(Context::builder()... .with_db(...), EthHandler::default())`
    pub fn new(creation_code: Vec<u8>) -> Result<Self> {
        // 1) Create the EVM using your old style: `Context::builder()...`
        //    Then pass to MainEvm::new(...).
        //    The difference is that we explicitly say MainEvm<DB,BLOCK,TX,CFG>.
        let mut evm = MainEvm::new(
            // a) Prepare context with a chained transaction to CREATE
            Context::builder()
                .modify_tx_chained(|tx| {
                    tx.transact_to = TxKind::Create;
                    tx.data = Bytes::from(creation_code.clone());
                })
                // b) Provide ephemeral DB
                .with_db(CacheDB::<EmptyDB>::default()),
            // c) The EthHandler for block/tx environment logic
            EthHandler::default(),
        );

        // 2) Execute the CREATE transaction
        let creation_result = evm.exec_commit()?;
        let ExecutionResult::Success {
            output: Output::Create(_, Some(deployed_addr)),
            ..
        } = creation_result
        else {
            bail!("Interpreter deployment failed or no address returned: {creation_result:#?}");
        };

        // 3) Return the EvmRunner
        Ok(Self {
            evm,
            interpreter_addr: deployed_addr,
        })
    }

    /// Call `runInterpreter(bytes,uint256[],uint256[],int256[])` on the deployed contract,
    /// returning the final code/exec/int stacks.
    pub fn run_interpreter(
        &mut self,
        inputs: &Push3InterpreterInputs
    ) -> Result<Push3InterpreterOutputs> {
        // 1) Build function selector
        let func_selector = &utils::id("runInterpreter(bytes,uint256[],uint256[],int256[],bool[])")[0..4];

        // 2) Convert each field to `ethers::abi::Token`
        let code_token = Token::Bytes(inputs.code.clone());
        let init_code_stack = Token::Array(
            inputs.init_code_stack
                .iter()
                .map(|&u| Token::Uint(u))
                .collect()
        );
        let init_exec_stack = Token::Array(
            inputs.init_exec_stack
                .iter()
                .map(|&u| Token::Uint(u))
                .collect()
        );
        let init_int_stack = Token::Array(
            inputs.init_int_stack
                .iter()
                .map(|&val_i128| {
                    // store i128 as a U256. If you need negative values, do a two’s complement approach.
                    let as_u128 = val_i128 as u128;
                    Token::Int(U256::from(as_u128))
                })
                .collect()
        );
        let init_bool_stack = Token::Array(
            inputs.init_bool_stack
                .iter()
                .map(|&b| Token::Bool(b))
                .collect()
        );

        // 3) Encode
        let encoded_args = encode(&[
            code_token,
            init_code_stack,
            init_exec_stack,
            init_int_stack,
            init_bool_stack,
        ]);

        // 4) Build final call data
        let mut call_data = Vec::from(func_selector);
        call_data.extend_from_slice(&encoded_args);

        // 5) Modify the transaction to CALL the deployed interpreter
        self.evm.context.modify_tx(|tx| {
            tx.transact_to = TxKind::Call(self.interpreter_addr);
            tx.data = Bytes::from(call_data);
            tx.nonce = 1; // increment nonce to avoid reuse
        });

        // 6) Execute the call
        let call_result = self.evm.transact()?;
        match &call_result.result {
            ExecutionResult::Success {
                output: Output::Call(return_data),
                ..
            } => {
                // 7) Decode (uint256[], uint256[], int256[], bool[])
                let param_types = &[
                    ParamType::Array(Box::new(ParamType::Uint(256))), // finalCodeStack
                    ParamType::Array(Box::new(ParamType::Uint(256))), // finalExecStack
                    ParamType::Array(Box::new(ParamType::Int(256))),  // finalIntStack
                    ParamType::Array(Box::new(ParamType::Bool)),      // finalBoolStack
                ];
                let decoded = decode(param_types, return_data)
                    .map_err(|e| anyhow!("Failed to decode return data: {e}"))?;

                // parse each array
                let final_code_stack = match &decoded[0] {
                    Token::Array(arr) => arr.iter().filter_map(|t| {
                        if let Token::Uint(u) = t { Some(*u) } else { None }
                    }).collect(),
                    _ => Vec::new(),
                };
                let final_exec_stack = match &decoded[1] {
                    Token::Array(arr) => arr.iter().filter_map(|t| {
                        if let Token::Uint(u) = t { Some(*u) } else { None }
                    }).collect(),
                    _ => Vec::new(),
                };
                let final_int_stack = match &decoded[2] {
                    Token::Array(arr) => arr.iter().filter_map(|t| {
                        if let Token::Int(u256_val) = t {
                            // read only the lower 128 bits => ignoring sign extension
                            let lo = u256_val.low_u128();
                            Some(lo as i128)
                        } else {
                            None
                        }
                    }).collect(),
                    _ => Vec::new(),
                };
                let final_bool_stack = match &decoded[3] {
                    Token::Array(arr) => arr.iter().filter_map(|t| {
                        if let Token::Bool(b) = t { Some(*b) } else { None }
                    }).collect(),
                    _ => Vec::new(),
                };

                Ok(Push3InterpreterOutputs {
                    final_code_stack,
                    final_exec_stack,
                    final_int_stack,
                    final_bool_stack,
                })
            }
            ExecutionResult::Revert { gas_used, output } => {
                bail!("Call reverted: gas used={gas_used:?}, output={output:?}")
            }
            other => {
                bail!("Call failed: {other:?}")
            }
        }
    }

    /// A convenience method to run an `UntypedAst`:
    /// - Convert AST => push3 code,
    /// - Build a sublist descriptor in the exec stack,
    /// - Call `run_interpreter`.
    pub fn run_ast(&mut self, ast: &UntypedAst) -> Result<Push3InterpreterOutputs> {
        // 1) Convert AST => push3 bytecode
        let code_bytes = ast.to_bytecode();
        let code_len = code_bytes.len() as u32;

        // 2) Build a sublist descriptor
        let descriptor = make_sublist_descriptor(0, code_len);

        // 3) Put code in `code` and sublist descriptor in `execStack`
        let inputs = Push3InterpreterInputs {
            code: code_bytes,
            init_code_stack: Vec::new(),
            init_exec_stack: vec![descriptor],
            init_int_stack: Vec::new(),
            init_bool_stack: Vec::new(),
        };

        // 4) Run interpreter
        self.run_interpreter(&inputs)
    }
}
