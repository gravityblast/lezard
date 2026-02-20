use anyhow::Result;
use lezard::{DEFAULT_PROGRAM_ID, deploy_program, get_account, send_unsigned_tx, wait_for_block};

fn read_u64(data: &[u8]) -> u64 {
    let arr: [u8; 8] = data.try_into().expect("account data is not a valid u64");
    u64::from_le_bytes(arr)
}

#[tokio::test]
async fn double_program() -> Result<()> {
    let ctx = lezard::test_setup().await?;
    let account_id = ctx.accounts[0];

    // Account starts uninitialized
    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(account.program_owner, DEFAULT_PROGRAM_ID);
    assert!(account.data.is_empty());

    // Deploy the double program
    let program_id = deploy_program(&ctx.client, "double").await?;
    wait_for_block(&ctx.client).await?;

    // First call: empty -> 1
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(&ctx.client).await?;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(account.program_owner, program_id);
    assert_eq!(read_u64(account.data.as_ref()), 1);

    // Second call: 1 -> 2
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(&ctx.client).await?;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(read_u64(account.data.as_ref()), 2);

    // Third call: 2 -> 4
    send_unsigned_tx(&ctx.client, program_id, vec![account_id], ()).await?;
    wait_for_block(&ctx.client).await?;

    let account = get_account(&ctx.client, account_id).await?;
    assert_eq!(read_u64(account.data.as_ref()), 4);

    Ok(())
}
