use revm::{Database, Evm, InMemoryDB};
use revm_primitives::{Address, U256, TxEnv, ExecutionResult, Output};
use crate::error::{CoreError, Result};

pub struct EvmState {
    db: InMemoryDB,
}

impl EvmState {
    pub fn new() -> Self {
        Self { db: InMemoryDB::default() }
    }

    pub fn deploy_contract(&mut self, deployer: Address, bytecode: Vec<u8>) -> Result<Address> {
        let mut evm = Evm::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|tx| {
                tx.caller = deployer;
                tx.data = bytecode.into();
                tx.transact_to = revm_primitives::TxKind::Create;
            })
            .build();

        match evm.transact_commit() {
            Ok(result) => match result {
                ExecutionResult::Success { output, .. } => {
                    if let Output::Create(_, Some(addr)) = output {
                        Ok(addr)
                    } else {
                        Err(CoreError::EvmError("Contract deployment failed".into()))
                    }
                }
                _ => Err(CoreError::EvmError("Execution failed".into())),
            },
            Err(e) => Err(CoreError::EvmError(format!("{:?}", e))),
        }
    }

    pub fn call_contract(&mut self, caller: Address, contract: Address, data: Vec<u8>) -> Result<Vec<u8>> {
        let mut evm = Evm::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|tx| {
                tx.caller = caller;
                tx.transact_to = revm_primitives::TxKind::Call(contract);
                tx.data = data.into();
            })
            .build();

        match evm.transact_commit() {
            Ok(result) => match result {
                ExecutionResult::Success { output, .. } => {
                    if let Output::Call(bytes) = output {
                        Ok(bytes.to_vec())
                    } else {
                        Ok(vec![])
                    }
                }
                _ => Err(CoreError::EvmError("Call failed".into())),
            },
            Err(e) => Err(CoreError::EvmError(format!("{:?}", e))),
        }
    }
}
