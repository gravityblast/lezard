use std::time::Duration;

use anyhow::{Context, Result};
use common::sequencer_client::SequencerClient;
use indexer_core::config::ChannelId;
use log::info;
use sequencer_core::config::{BedrockConfig, SequencerConfig};
use sequencer_runner::SequencerHandle;
use url::Url;

pub use common::sequencer_client;
pub use nssa::{
    Account, AccountId, ProgramDeploymentTransaction, PublicTransaction,
    program::Program,
    program_deployment_transaction::Message as DeployMessage,
    public_transaction::{Message, WitnessSet},
};
pub use nssa_core::program::{DEFAULT_PROGRAM_ID, ProgramId};

/// Sequencer block time in milliseconds
const BLOCK_TIME_MS: u64 = 200;

/// The milliseconds to wait for the next block in wait_for_block before timing out.
const NEXT_BLOCK_TIMEOUT_MS: u64 = 2000;

/// Number of pre-generated accounts in `LezardContext::accounts`.
const NUM_ACCOUNTS: usize = 10;

/// Initializes logging and starts a local sequencer.
/// For now it must be the first call in every `#[tokio::test]`.
///
/// Logging defaults to `warn,lezard=info` so only framework messages show.
/// Override with `RUST_LOG=debug` (or `RUST_LOG=info`) for full output.
///
/// Uses `try_init()` so multiple tests in the same process don't panic.
pub async fn test_setup() -> Result<LezardContext> {
    let _ = env_logger::builder()
        .parse_env(env_logger::Env::default().default_filter_or("warn,lezard=info"))
        .try_init();
    start_sequencer().await
}

/// Bundles a running sequencer with its client and temp storage.
pub struct LezardContext {
    pub handle: SequencerHandle,
    pub client: SequencerClient,
    /// 10 pre-generated account IDs, ready to use in tests.
    /// `accounts[0]` = `[1; 32]`, `accounts[1]` = `[2; 32]`, ..., `accounts[9]` = `[10; 32]`.
    pub accounts: [AccountId; NUM_ACCOUNTS],
    _tmp_dir: tempfile::TempDir,
}

/// Starts a standalone sequencer (no Docker/Bedrock needed) with an empty
/// genesis state. Returns a `LezardContext` containing the handle and client.
pub async fn start_sequencer() -> Result<LezardContext> {
    let tmp_dir = tempfile::tempdir().context("Failed to create temp dir")?;

    let config = SequencerConfig {
        home: tmp_dir.path().to_path_buf(),
        override_rust_log: None,
        genesis_id: 1,
        is_genesis_random: true,
        max_num_tx_in_block: 20,
        mempool_max_size: 10_000,
        block_create_timeout_millis: BLOCK_TIME_MS,
        retry_pending_blocks_timeout_millis: 240_000,
        port: 0,
        initial_accounts: vec![],
        initial_commitments: vec![],
        signing_key: [37; 32],
        bedrock_config: BedrockConfig {
            backoff: Default::default(),
            channel_id: ChannelId::from([0u8; 32]),
            node_url: "http://127.0.0.1:1".parse::<Url>().unwrap(),
            auth: None,
        },
        indexer_rpc_url: "ws://127.0.0.1:1".parse::<Url>().unwrap(),
    };

    info!("Starting standalone sequencer...");
    let handle = sequencer_runner::startup_sequencer(config)
        .await
        .context("Failed to start sequencer")?;

    let addr = handle.addr();
    info!("Sequencer listening on {addr}");

    let url: Url = format!("http://127.0.0.1:{}", addr.port()).parse()?;
    let client = SequencerClient::new(url)?;

    let accounts = std::array::from_fn(|i| AccountId::new([(i + 1) as u8; 32]));

    Ok(LezardContext {
        handle,
        client,
        accounts,
        _tmp_dir: tmp_dir,
    })
}

/// Deploys a guest program to the sequencer by name.
/// The name corresponds to the binary name in `programs/src/bin/` (e.g. `"double"`).
/// Returns the `ProgramId` assigned to the deployed program.
pub async fn deploy_program(client: &SequencerClient, name: &str) -> Result<ProgramId> {
    let elf_path = std::env::current_dir()?
        .join("target/riscv32im-risc0-zkvm-elf/docker")
        .join(format!("{name}.bin"));

    let bytecode = std::fs::read(&elf_path)
        .with_context(|| format!("Failed to read ELF binary at {}", elf_path.display()))?;
    info!(
        "Read ELF ({} bytes) from {}",
        bytecode.len(),
        elf_path.display()
    );

    let program = Program::new(bytecode.clone())
        .context("Failed to parse ELF as a valid RISC Zero program")?;
    let program_id = program.id();
    info!("Program ID: {program_id:?}");

    let deploy_tx = ProgramDeploymentTransaction::new(DeployMessage::new(bytecode));
    let resp = client
        .send_tx_program(deploy_tx)
        .await
        .context("Failed to send program deployment transaction")?;
    info!("Deploy TX sent: {}", resp.status);

    Ok(program_id)
}

/// Sends a public transaction that invokes `program_id` with the given
/// accounts, instruction data, and no signers.
pub async fn send_unsigned_tx(
    client: &SequencerClient,
    program_id: ProgramId,
    account_ids: Vec<AccountId>,
    instruction: impl serde::Serialize,
) -> Result<()> {
    let message = Message::try_new(program_id, account_ids, vec![], instruction)
        .context("Failed to create message")?;
    let witness_set = WitnessSet::for_message(&message, &[]);
    let tx = PublicTransaction::new(message, witness_set);

    let resp = client
        .send_tx_public(tx)
        .await
        .context("Failed to send public transaction")?;
    info!("TX sent: {}", resp.status);

    Ok(())
}

/// Fetches the current on-chain state of an account.
pub async fn get_account(client: &SequencerClient, account_id: AccountId) -> Result<Account> {
    let resp = client
        .get_account(account_id)
        .await
        .context("Failed to get account")?;
    Ok(resp.account)
}

/// Waits until a new block is produced after the current one.
/// Polls `get_last_block()` every BLOCK_TIME_MS, with a timeout based on NEXT_BLOCK_TIMEOUT_MS.
pub async fn wait_for_block(client: &SequencerClient) -> Result<u64> {
    let initial = client
        .get_last_block()
        .await
        .context("Failed to get last block")?
        .last_block;

    let timeout = Duration::from_millis(NEXT_BLOCK_TIMEOUT_MS);
    let start = tokio::time::Instant::now();

    loop {
        tokio::time::sleep(Duration::from_millis(BLOCK_TIME_MS)).await;

        let current = client
            .get_last_block()
            .await
            .context("Failed to get last block")?
            .last_block;

        if current > initial {
            info!("Block {current} produced");
            return Ok(current);
        } else {
            info!("Waiting for the next block...")
        }

        if start.elapsed() > timeout {
            anyhow::bail!(
                "Timed out waiting for new block after {timeout:?} (stuck at block {initial})"
            );
        }
    }
}
