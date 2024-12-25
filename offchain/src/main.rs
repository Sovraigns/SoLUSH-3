mod compiler;

use std::env;
use compiler::ast::{
    parse_string_to_sexpr,
    sexpr_to_untyped,
    Push3Ast, // The trait
    UntypedAst, // The untyped AST enum
};

use anyhow::{anyhow, bail, Result};
use hex;
use database::CacheDB;
use revm::{
    context::Context,
    context_interface::result::{ExecutionResult, Output},
    database_interface::EmptyDB,
    handler::EthHandler,
    primitives::{Bytes, TxKind},
    EvmCommit, MainEvm,
};
use ethers::abi::{encode, Token};
use ethers::abi::{ParamType, decode};
use ethers::types::U256;               // Use ethers' U256 instead



fn main() -> Result<()> {
    
    // 1) Read CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- '<program>'");
        eprintln!("Example: cargo run -- '((3 5 +) DUP MUL)'");
        // If you want to indicate a failure exit code:
        std::process::exit(1);
    }
    let program_str = &args[1];

    // 2) Parse the string into an S-expression
    let sexpr = parse_string_to_sexpr(program_str)
        .map_err(|e| anyhow::anyhow!("Error parsing S-expression: {}", e))?;

    // 3) Convert SExpr => UntypedAst
    let ast: UntypedAst = sexpr_to_untyped(&sexpr)
        .map_err(|e| anyhow::anyhow!("Error converting to UntypedAst: {}", e))?;

    // 4) Convert UntypedAst => Push3 bytecode
    let bytecode = ast.to_bytecode();

    // 5) Print the resulting AST and bytecode (in hex)
    println!("AST: {:?}", ast);
    println!("Bytecode (hex): 0x{}", hex::encode(&bytecode));
    
    // 1) Suppose you have compiled your contract with Forge.
    //    Grab the "bytecode" field from e.g. `out/MyContract.sol/MyContract.json`.
    //    We'll store it here as a string. This is your entire "creation code."
    //    (Example placeholder below.)
    let creation_hex = "6080806040523461001657610cd8908161001b8239f35b5f80fdfe6080604052600480361015610012575f80fd5b5f803560e01c918263093b62c0146100395750506336d9ea8514610034575f80fd5b6101ce565b346101045760803660031901126101045780359167ffffffffffffffff918284116101045736602385011215610104578381013590838211610100573660248387010111610100576024358481116100fc576100989036908301610108565b9290916044358681116100f8576100b29036908301610108565b9690956064359182116100f5576100f16100e260248b8b8b8b8b8b6100d9368c8e01610108565b97909601610480565b60409391935193849384610170565b0390f35b80fd5b8580fd5b8380fd5b8280fd5b5080fd5b9181601f840112156101395782359167ffffffffffffffff8311610139576020808501948460051b01011161013957565b5f80fd5b9081518082526020808093019301915f5b82811061015c575050505090565b83518552938101939281019260010161014e565b906101866101959160608452606084019061013d565b6020938382038585015261013d565b9060408183039101528180845192838152019301915f5b8281106101ba575050505090565b8351855293810193928101926001016101ac565b346101395760803660031901126101395760043560048110156101395763ffffffff60243581811681036101395760443591821682036101395760643590600160b81b82101561024c576020936040519363ffffffff60b81b9060b81b169163ffffffff60d81b9060d81b169060ff60f81b9060f81b161717178152f35b60405162461bcd60e51b81526020600482015260136024820152726c6f77313834206f7574206f662072616e676560681b6044820152606490fd5b634e487b7160e01b5f52602160045260245ffd5b600411156102a557565b610287565b600160b81b81101561024c57600160f91b1790565b634e487b7160e01b5f52601160045260245ffd5b9061010082018092116102e257565b6102bf565b634e487b7160e01b5f52604160045260245ffd5b67ffffffffffffffff81116103135760051b60200190565b6102e7565b90610322826102fb565b60405190601f1990601f018116820167ffffffffffffffff8111838210176103135760405283825261035482946102fb565b0190602036910137565b5f1981146102e25760010190565b634e487b7160e01b5f52603260045260245ffd5b91908110156103905760051b0190565b61036c565b80518210156103905760209160051b010190565b5f198101919082116102e257565b6001198101919082116102e257565b919082039182116102e257565b80156102e2575f190190565b90600263ffffffff809316019182116102e257565b90600463ffffffff809316019182116102e257565b91909163ffffffff808094169116019182116102e257565b600611156102a557565b81810292915f8212600160ff1b8214166102e25781840514901517156102e257565b81810392915f1380158285131691841216176102e257565b9190915f83820193841291129080158216911516176102e257565b9794929197969396610499610494846102d3565b610318565b985f5b8481106108ac5750506104b1610494856102d3565b97845f5b81811061088c57505050836104cc610494876102d3565b97865f5b818110610866575050505b6105855750506104ea81610318565b965f5b8281106105675750505061050081610318565b945f5b8281106105495750505061051681610318565b925f5b82811061052557505050565b806105336105449284610395565b5161053e8288610395565b5261035e565b610519565b806105576105629284610395565b5161053e828a610395565b610503565b806105756105809284610395565b5161053e828c610395565b6104ed565b6105a161059a610594866103a9565b8a610395565b51946103d3565b93846105af8260f81c6108c5565b916105b98361029b565b600192808403610777575060ff6105d091166108d4565b916105da83610421565b826105e8575b5090506104db565b6105f183610421565b82810361065c57505050600285101561060e575b835b805f6105e0565b6106558561064b61062861062288996103a9565b8b610395565b5161064561063e610638856103b7565b8d610395565b51936103b7565b92610465565b61053e828b610395565b9450610605565b61066583610421565b6002928084036106b157505050851015610680575b83610607565b6106aa8561064b61069461062288996103a9565b516106a461063e610638856103b7565b9261044d565b945061067a565b6106ba81610421565b600381036106fc575050508510156106d25783610607565b6106aa8561064b6106e661062288996103a9565b516106f661063e610638856103b7565b9261042b565b9091925061070981610421565b6004810361073d575090508510156107215783610607565b6106aa8561073261059487986103a9565b5161053e828b610395565b80610749600592610421565b14610755575b50610607565b9050851015610766575b835f61074f565b61077084956103a9565b945061075f565b909291506107848161029b565b600281036107ad575050849563ffffffff6107a7921660030b61053e828b610395565b946104db565b916003836107bb899561029b565b0361085f5763ffffffff908582808360d81c169260b81c16926107de8484610409565b1611156107ee575b5050506104db565b829350906107fd918686610904565b5f925b61080c575b86926107e6565b9091958a825180891015610857578261084f9261053e879561084261083c8e610837610849986103a9565b6103c6565b89610395565b5192610395565b9761035e565b929190610800565b505095610805565b50506104db565b82935061087881836108839495610380565b3561053e828d610395565b908692916104d0565b808b61053e826108a06108a7958789610380565b3592610395565b6104b5565b808b61053e826108a06108c0958a88610380565b61049c565b60ff1660048110156102a55790565b60ff1660068110156102a55790565b90821015610390570190565b63ffffffff8091169081146102e25760010190565b919063ffffffff92610917848616610318565b95846109245f9786610409565b16955b85851687811015610bbd5784811015610bbd5761096a61096461095e61095060ff9489896108e3565b356001600160f81b03191690565b60f81c90565b966108ef565b95168061098d575061098590600160f81b61053e828b610395565b955b95610927565b600181036109b257506109ac906001600160f81b0161053e828b610395565b95610987565b60028103610a495750866109d16109c8876103f4565b63ffffffff1690565b111580610a34575b15610a01576109ac9061064b876109fa6109f4898989610bca565b986103f4565b97166102aa565b945094505050505b610a1281610318565b925f5b828110610a2157505050565b80610533610a2f9284610395565b610a15565b5083610a426109c8876103f4565b11156109d9565b9096949060038103610b2c575084610a636109c8836103df565b111580610b17575b15610b0b57610a84610a7e828686610c36565b916103df565b9661ffff82169186610a996109c8858c610409565b111580610af5575b15610ae757610ae19291610adb9160d88b901b63ffffffff60d81b1660b89190911b61ffff60b81b1617600360f81b1761053e828d610395565b97610409565b93610987565b509550509450505050610a09565b5085610b046109c8858c610409565b1115610aa1565b50949350505050610a09565b5083610b256109c8836103df565b1115610a6b565b9496909460048103610b4f57506109ac906002600160f81b0161053e828b610395565b60058103610b6e57506109ac906003600160f81b0161053e828b610395565b60068103610b8d57506109ac906004600160f81b0161053e828b610395565b600703610baa576109ac906005600160f81b0161053e828b610395565b6109ac90600160f81b61053e828b610395565b5094509450505050610a09565b919063ffffffff610bda836103f4565b1611610bf157600490604051920182375160e01c90565b60405162461bcd60e51b815260206004820152601760248201527f7265616455696e743332206f7574206f662072616e67650000000000000000006044820152606490fd5b919063ffffffff610c46836103df565b1611610c5d57600290604051920182375160f01c90565b60405162461bcd60e51b815260206004820152601760248201527f7265616455696e743136206f7574206f662072616e67650000000000000000006044820152606490fdfea26469706673582212208a8e0c1517585160ff23b605caf6916a05dae2b541b9387b0dfab23ea71f563964736f6c63430008140033"; 
    // ^^^^^ Replace with your actual creation code from forge.

    // 2) Convert hex string to raw bytes.
    let creation_bytes = match hex::decode(creation_hex) {
        Ok(bytes) => bytes,
        Err(e) => bail!("Invalid hex for creation code: {e}"),
    };

    // 3) Build a new EVM instance with a "Create" transaction:
    let mut evm = MainEvm::new(
        // a) Prepare context with a chained transaction
        Context::builder()
            .modify_tx_chained(|tx| {
                tx.transact_to = TxKind::Create;
                tx.data = Bytes::from(creation_bytes.clone()); 
                // If you want to attach a value, e.g. tx.value = <some U256>
            })
            // b) Provide an in-memory DB for ephemeral state
            .with_db(CacheDB::<EmptyDB>::default()),
        // c) The "EthHandler" for block/tx environment logic
        EthHandler::default(),
    );

    println!("Deploying contract with creation code size: {} bytes", creation_bytes.len());

    // 4) Execute the CREATE transaction
    let creation_result = evm.exec_commit()?; 
    // => an ExecutionResult

    // 5) Check success
    let ExecutionResult::Success {
        output: Output::Create(_, Some(deployed_addr)),
        ..
    } = creation_result
    else {
        bail!("Deployment failed or didn't return an address: {creation_result:#?}");
    };

    println!("Deployed contract at: 0x{:x}", deployed_addr);
    
    // 1) Prepare the function selector for runInterpreter
    let func_selector = &ethers::utils::id("runInterpreter(bytes,uint256[],uint256[],int256[])")[0..4];
    
    // 2) Convert your compiled “Push3” bytecode into the `bytes` argument
    let code_arg = Token::Bytes(bytecode.clone());
    
    // 3) Build a “SUBLIST descriptor” for the entire bytecode.
    //    In Solidity, SUBLIST = CodeTag.SUBLIST = 3. The layout is:
    //    [255..248]=tag, [247..216]=offset, [215..184]=length, [183..0]=low184
    let tag_bits    = U256::from(3u64)                  << 248; // CodeTag.SUBLIST = 3
    let offset_bits = U256::from(0u64)                  << 216; // offset=0
    let length_bits = U256::from(bytecode.len() as u64) << 184; // length = bytecode.len()
    let low_184     = U256::zero();                                
    
    let sublist_descriptor = tag_bits | offset_bits | length_bits | low_184;

    // Now we put *that* descriptor as the one and only item in `initExecStack`.
    let init_exec_stack = Token::Array(vec![
        Token::Uint(sublist_descriptor)
    ]);
    
    // For the other two arrays (codeStack, intStack), we’ll just leave them empty:
    let empty_uint256_array = Token::Array(vec![]); // codeStack
    let empty_int256_array  = Token::Array(vec![]); // intStack

    // 4) Encode all arguments in the correct order:
    //    (bytes code, uint256[] initCodeStack, uint256[] initExecStack, int256[] initIntStack)
    let encoded_args = encode(&[
        code_arg,
        empty_uint256_array.clone(),   // initCodeStack
        init_exec_stack,               // initExecStack
        empty_uint256_array.clone(),   // initIntStack (or empty_int256_array if needed)
    ]);
    
    // 5) Concatenate selector + encoded arguments to form the full call data
    let mut call_data = Vec::from(func_selector);
    call_data.extend_from_slice(&encoded_args);
    
    // 6) Modify the transaction to call `runInterpreter` with the encoded data
    evm.context.modify_tx(|tx| {
        tx.transact_to = TxKind::Call(deployed_addr);
        tx.data = Bytes::from(call_data);
        tx.nonce += 1; // increment nonce to avoid reuse
    });
    
    // 7) Execute the call.
    let call_result = evm.transact()?;
    match &call_result.result {
        ExecutionResult::Success {
            output: Output::Call(return_data),
            ..
        } => {
            println!("Call succeeded, returned data: 0x{}", hex::encode(return_data));
            
            // Decode (uint256[], uint256[], int256[])
            use ethers::abi::{decode, ParamType};
            
            let param_types = &[
                ParamType::Array(Box::new(ParamType::Uint(256))), // finalCodeStack
                ParamType::Array(Box::new(ParamType::Uint(256))), // finalExecStack
                ParamType::Array(Box::new(ParamType::Int(256))),  // finalIntStack
            ];
            
            let decoded = decode(param_types, return_data)
                .map_err(|e| anyhow!("Failed to decode return data: {e}"))?;
            
            println!("Decoded return: {:?}", decoded);
        }
        ExecutionResult::Revert { gas_used, output } => {
            bail!("Call reverted: gas used = {gas_used:?}, output = {:?}", output);
        }
        other => {
            bail!("Call failed: {other:?}");
        }
    }    
    Ok(())
}
