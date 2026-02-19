use std::{path::PathBuf, time::Duration};

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

/// Bundles a running sequencer with its client and temp storage.
pub struct LezardContext {
    pub handle: SequencerHandle,
    pub client: SequencerClient,
    pub block_timeout_ms: u64,
    _tmp_dir: tempfile::TempDir,
}

/// Starts a standalone sequencer (no Docker/Bedrock needed) with an empty
/// genesis state. Returns a `LezardContext` containing the handle and client.
pub async fn start_sequencer(block_timeout_ms: u64) -> Result<LezardContext> {
    let tmp_dir = tempfile::tempdir().context("Failed to create temp dir")?;

    let config = SequencerConfig {
        home: tmp_dir.path().to_path_buf(),
        override_rust_log: None,
        genesis_id: 1,
        is_genesis_random: true,
        max_num_tx_in_block: 20,
        mempool_max_size: 10_000,
        block_create_timeout_millis: block_timeout_ms,
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

    Ok(LezardContext {
        handle,
        client,
        block_timeout_ms,
        _tmp_dir: tmp_dir,
    })
}

/// Reads an ELF binary from disk and deploys it to the sequencer.
/// Returns the `ProgramId` assigned to the deployed program.
pub async fn deploy_program(client: &SequencerClient, elf_path: &PathBuf) -> Result<ProgramId> {
    let elf_path = if elf_path.is_relative() {
        std::env::current_dir()?.join(elf_path)
    } else {
        elf_path.clone()
    };

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

/// Sleeps long enough for at least one block to be produced.
pub async fn wait_for_block(block_timeout_ms: u64) {
    let wait = Duration::from_millis(block_timeout_ms + 1000);
    info!("Waiting {wait:?} for next block...");
    tokio::time::sleep(wait).await;
}
