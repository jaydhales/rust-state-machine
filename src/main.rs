use crate::{
    types::{AccountId, Balance, Block, BlockNumber, Extrinsic, Header, Nonce},
};
use crate::support::Dispatch;

mod balances;
mod proof_of_existence;
mod support;
mod system;

mod types {
    use crate::{RuntimeCall, support};

    pub type AccountId = String;
    pub type Balance = u128;
    pub type BlockNumber = u32;
    pub type Nonce = u32;
    pub type Extrinsic = support::Extrinsic<AccountId, RuntimeCall>;
    pub type Header = support::Header<BlockNumber>;
    pub type Block = support::Block<Header, Extrinsic>;
    pub type Content = String;
}

#[derive(Clone, Debug)]
pub enum RuntimeCall {
    Balances(balances::Call<Runtime>),
    ProofOfExistence(proof_of_existence::Call<Runtime>),
}

// This is our main Runtime.
// It accumulates all the different pallets we want to use.
#[derive(Clone, Debug)]
pub struct Runtime {
    system: system::Pallet<Runtime>,
    balances: balances::Pallet<Runtime>,
    proof_of_existence: proof_of_existence::Pallet<Runtime>,
}

impl system::Config for Runtime {
    type AccountId = AccountId;
    type BlockNumber = BlockNumber;
    type Nonce = Nonce;
}

impl balances::Config for Runtime {
    type Balance = Balance;
}

impl proof_of_existence::Config for Runtime {
    type Content = String;
}

impl Runtime {
    // Create a new instance of the main Runtime, by creating a new instance of each pallet.
    fn new() -> Self {
        Self {
            system: system::Pallet::new(),
            balances: balances::Pallet::new(),
            proof_of_existence: proof_of_existence::Pallet::new()
        }
    }

    fn execute_block(&mut self, block: Block) -> support::DispatchResult {
        self.system.inc_block_number();
        if block.header.block_number != self.system.block_number() {
            return Err("block number does not match what is expected");
        };
        for (i, Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
            self.system.inc_nonce(&caller);
            let _ = self.dispatch(caller, call).map_err(|e| {
                eprintln!(
                    "Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
                    block.header.block_number, i, e
                )
            });
        }

        Ok(())
    }
}

impl support::Dispatch for Runtime {
    type Caller = <Runtime as system::Config>::AccountId;
    type Call = RuntimeCall;
    // Dispatch a call on behalf of a caller. Increments the caller's nonce.
    //
    // Dispatch allows us to identify which underlying module call we want to execute.
    // Note that we extract the `caller` from the extrinsic, and use that information
    // to determine who we are executing the call on behalf of.
    fn dispatch(
        &mut self,
        caller: Self::Caller,
        runtime_call: Self::Call,
    ) -> support::DispatchResult {
        match runtime_call {
            RuntimeCall::Balances(call) => self.balances.dispatch(caller, call),
            RuntimeCall::ProofOfExistence(call) => self.proof_of_existence.dispatch(caller, call)
        }
    }
}

fn main() {
    let mut runtime = Runtime::new();

    let alice = String::from("alice");
    let bob = String::from("bob");
    let charlie = String::from("charlie");
    runtime.balances.set_balance(&alice, 100);

    let current_block = runtime.system.block_number();


    let new_block = Block {
        header: Header {
            block_number: current_block + 1,
        },
        extrinsics: vec![ Extrinsic {
            caller: alice.clone(),
            call: RuntimeCall::Balances(balances::Call::Transfer {
                to: bob,
                amount: 30,
            }),
        }, Extrinsic {
            caller: alice.clone(),
            call: RuntimeCall::Balances(balances::Call::Transfer {
                to: charlie.clone(),
                amount: 30,
            }),
        }],
    };

    println!("Block 1: {:?}", new_block);

    let _ = runtime
        .execute_block(new_block)
        .map_err(|e| eprintln!("{e}"));

    let current_block = runtime.system.block_number();
    let new_block = Block {
        header: Header {
            block_number: current_block + 1,
        },
        extrinsics: vec![
            Extrinsic {
                caller: charlie.clone(),
                call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
                    claim: String::from("I did that")
                })
            }
        ],
    };

    println!("Block 2: {:?}", new_block);

    let _ = runtime
        .execute_block(new_block)
        .map_err(|e| eprintln!("{e}"));

    println!("Runtime: {:?}", runtime);
}

