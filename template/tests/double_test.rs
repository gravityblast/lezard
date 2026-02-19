use std::path::PathBuf;

use anyhow::Result;
use lezard::{
    AccountId, DEFAULT_PROGRAM_ID, deploy_program, get_account, send_unsigned_tx, start_sequencer,
    wait_for_block,
};

fn read_u64(data: &[u8]) -> u64 {
    let arr: [u8; 8] = data.try_into().expect("account data is not a valid u64");
    u64::from_le_bytes(arr)
}

#[tokio::test]
async fn double_program() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let block_timeout_ms: u64 = 2000;
    let ctx = start_sequencer(block_timeout_ms).await?;

    let account_id = AccountId::new([42; 32]);

    // Account starts uninitialized
    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(account.program_owner, DEFAULT_PROGRAM_ID);
    assert!(account.data.is_empty());

    // Deploy the double program
    let elf_path = PathBuf::from("target/riscv32im-risc0-zkvm-elf/docker/double.bin");
    let program_id = deploy_program(&ctx.client, &elf_path).await?;
    wait_for_block(block_timeout_ms).await;

    // First call: empty -> 1
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(block_timeout_ms).await;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(account.program_owner, program_id);
    assert_eq!(read_u64(account.data.as_ref()), 1);

    // Second call: 1 -> 2
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(block_timeout_ms).await;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(read_u64(account.data.as_ref()), 2);

    // Third call: 2 -> 4
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(block_timeout_ms).await;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(read_u64(account.data.as_ref()), 4);

    Ok(())
}
