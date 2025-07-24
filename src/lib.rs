#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

mod dapps;
mod pb;
mod utils;
mod trade_instruction;

use pb::sf::solana::dex::trades::v1::{ Output, TradeData };
use substreams::log;
use substreams_solana::pb::sf::solana::r#type::v1::{
    Block,
    InnerInstructions,
    TokenBalance,
    ConfirmedTransaction,
};
use utils::get_mint;
use utils::{ convert_to_date, get_amt };

#[substreams::handlers::map]
fn map_block(block: Block) -> Result<Output, substreams::errors::Error> {
    process_block(block)
}

fn process_block(block: Block) -> Result<Output, substreams::errors::Error> {
    let slot = block.slot;
    let parent_slot = block.parent_slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<TradeData> = vec![];
    if timestamp.is_some() {
        let timestamp = timestamp.unwrap().timestamp;
        for trx in block.transactions_owned() {
            let accounts = resolved_accounts_as_strings(&trx);

            if let Some(transaction) = trx.transaction {
                let meta = trx.meta.unwrap();
                let pre_balances = meta.pre_balances;
                let post_balances = meta.post_balances;
                let pre_token_balances = meta.pre_token_balances;
                let post_token_balances = meta.post_token_balances;

                let msg = transaction.message.unwrap();

                for (idx, inst) in msg.instructions.into_iter().enumerate() {
                    let inner_instructions: Vec<InnerInstructions> = filter_inner_instructions(
                        &meta.inner_instructions,
                        idx as u32
                    );

                    let program = &accounts[inst.program_id_index as usize];
                    let trade_data = get_trade_instruction(
                        program,
                        inst.data,
                        &inst.accounts,
                        &accounts,
                        &pre_token_balances,
                        &post_token_balances,
                        &"".to_string(),
                        false,
                        &inner_instructions,
                        0 as u32
                    );
                    if trade_data.is_some() {
                        let td = trade_data.unwrap();

                        let td_name = td.name;
                        let td_dapp_address = td.dapp_address;

                        data.push(TradeData {
                            block_date: convert_to_date(timestamp),
                            tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                            block_slot: slot,
                            block_time: timestamp,
                            signer: accounts.get(0).unwrap().to_string(),
                            pool_address: td.amm,
                            base_mint: get_mint(
                                &td.vault_a,
                                &post_token_balances,
                                &accounts,
                                td_dapp_address.clone()
                            ),
                            quote_mint: get_mint(
                                &td.vault_b,
                                &post_token_balances,
                                &accounts,
                                "".to_string()
                            ),
                            base_amount: get_amt(
                                &td.vault_a,
                                0 as u32,
                                &inner_instructions,
                                &accounts,
                                &post_token_balances,
                                td_dapp_address.clone(),
                                pre_balances.clone(),
                                post_balances.clone(),
                                td.fee_account.clone()
                            ),
                            quote_amount: get_amt(
                                &td.vault_b,
                                0 as u32,
                                &inner_instructions,
                                &accounts,
                                &post_token_balances,
                                "".to_string(),
                                pre_balances.clone(),
                                post_balances.clone(),
                                td.fee_account.clone()
                            ),
                            base_vault: td.vault_a,
                            quote_vault: td.vault_b,
                            is_inner_instruction: false,
                            instruction_index: idx as u32,
                            instruction_type: td_name.clone(),
                            inner_instruxtion_index: 0,
                            outer_program: td_dapp_address.clone(),
                            inner_program: "".to_string(),
                            txn_fee_lamports: meta.fee,
                            signer_lamports_change: get_signer_balance_change(
                                &pre_balances,
                                &post_balances
                            ),
                        });

                        if td.second_swap_amm.clone().unwrap_or_default() != "" {
                            data.push(TradeData {
                                block_date: convert_to_date(timestamp),
                                tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                                block_slot: slot,
                                block_time: timestamp,
                                signer: accounts.get(0).unwrap().to_string(),
                                pool_address: td.second_swap_amm.clone().unwrap(),
                                base_mint: get_mint(
                                    &td.second_swap_vault_a.clone().unwrap(),
                                    &post_token_balances,
                                    &accounts,
                                    "".to_string()
                                ),
                                quote_mint: get_mint(
                                    &td.second_swap_vault_b.clone().unwrap(),
                                    &post_token_balances,
                                    &accounts,
                                    "".to_string()
                                ),
                                base_amount: get_amt(
                                    &td.second_swap_vault_a.clone().unwrap(),
                                    0 as u32,
                                    &inner_instructions,
                                    &accounts,
                                    &post_token_balances,
                                    "".to_string(),
                                    pre_balances.clone(),
                                    post_balances.clone(),
                                    td.fee_account.clone()
                                ),
                                quote_amount: get_amt(
                                    &td.second_swap_vault_b.clone().unwrap(),
                                    0 as u32,
                                    &inner_instructions,
                                    &accounts,
                                    &post_token_balances,
                                    "".to_string(),
                                    pre_balances.clone(),
                                    post_balances.clone(),
                                    td.fee_account.clone()
                                ),
                                base_vault: td.second_swap_vault_a.clone().unwrap(),
                                quote_vault: td.second_swap_vault_b.clone().unwrap(),
                                is_inner_instruction: false,
                                instruction_index: idx as u32,
                                instruction_type: td_name.clone(),
                                inner_instruxtion_index: 0,
                                outer_program: td_dapp_address.clone(),
                                inner_program: "".to_string(),
                                txn_fee_lamports: meta.fee,
                                signer_lamports_change: get_signer_balance_change(
                                    &pre_balances,
                                    &post_balances
                                ),
                            });
                        }
                    }

                    meta.inner_instructions
                        .iter()
                        .filter(|inner_instruction| inner_instruction.index == (idx as u32))
                        .for_each(|inner_instruction| {
                            inner_instruction.instructions
                                .iter()
                                .enumerate()
                                .for_each(|(inner_idx, inner_inst)| {
                                    let inner_program =
                                        &accounts[inner_inst.program_id_index as usize];
                                    let inner_trade_data = get_trade_instruction(
                                        inner_program,
                                        inner_inst.data.clone(),
                                        &inner_inst.accounts,
                                        &accounts,
                                        &pre_token_balances,
                                        &post_token_balances,
                                        &program.to_string(),
                                        true,
                                        &inner_instructions,
                                        inner_idx as u32
                                    );

                                    if inner_trade_data.is_some() {
                                        let inner_td = inner_trade_data.unwrap();

                                        let inner_td_name = inner_td.name;
                                        let inner_td_dapp_address = inner_td.dapp_address;

                                        data.push(TradeData {
                                            block_date: convert_to_date(timestamp),
                                            tx_id: bs58
                                                ::encode(&transaction.signatures[0])
                                                .into_string(),
                                            block_slot: slot,
                                            block_time: timestamp,
                                            signer: accounts.get(0).unwrap().to_string(),
                                            pool_address: inner_td.amm,
                                            base_mint: get_mint(
                                                &inner_td.vault_a,
                                                &post_token_balances,
                                                &accounts,
                                                inner_td_dapp_address.clone()
                                            ),
                                            quote_mint: get_mint(
                                                &inner_td.vault_b,
                                                &post_token_balances,
                                                &accounts,
                                                "".to_string()
                                            ),
                                            base_amount: get_amt(
                                                &inner_td.vault_a,
                                                inner_idx as u32,
                                                &inner_instructions,
                                                &accounts,
                                                &post_token_balances,
                                                inner_td_dapp_address.clone(),
                                                pre_balances.clone(),
                                                post_balances.clone(),
                                                inner_td.fee_account.clone()
                                            ),
                                            quote_amount: get_amt(
                                                &inner_td.vault_b,
                                                inner_idx as u32,
                                                &inner_instructions,
                                                &accounts,
                                                &post_token_balances,
                                                "".to_string(),
                                                pre_balances.clone(),
                                                post_balances.clone(),
                                                inner_td.fee_account.clone()
                                            ),
                                            base_vault: inner_td.vault_a,
                                            quote_vault: inner_td.vault_b,
                                            is_inner_instruction: true,
                                            instruction_index: idx as u32,
                                            instruction_type: inner_td_name.clone(),
                                            inner_instruxtion_index: inner_idx as u32,
                                            outer_program: program.to_string(),
                                            inner_program: inner_td_dapp_address.clone(),
                                            txn_fee_lamports: meta.fee,
                                            signer_lamports_change: get_signer_balance_change(
                                                &pre_balances,
                                                &post_balances
                                            ),
                                        });

                                        if
                                            inner_td.second_swap_amm.clone().unwrap_or_default() !=
                                            ""
                                        {
                                            data.push(TradeData {
                                                block_date: convert_to_date(timestamp),
                                                tx_id: bs58
                                                    ::encode(&transaction.signatures[0])
                                                    .into_string(),
                                                block_slot: slot,
                                                block_time: timestamp,
                                                signer: accounts.get(0).unwrap().to_string(),
                                                pool_address: inner_td.second_swap_amm
                                                    .clone()
                                                    .unwrap(),
                                                base_mint: get_mint(
                                                    &inner_td.second_swap_vault_a.clone().unwrap(),
                                                    &post_token_balances,
                                                    &accounts,
                                                    "".to_string()
                                                ),
                                                quote_mint: get_mint(
                                                    &inner_td.second_swap_vault_b.clone().unwrap(),
                                                    &post_token_balances,
                                                    &accounts,
                                                    "".to_string()
                                                ),
                                                base_amount: get_amt(
                                                    &inner_td.second_swap_vault_a.clone().unwrap(),
                                                    inner_idx as u32,
                                                    &inner_instructions,
                                                    &accounts,
                                                    &post_token_balances,
                                                    "".to_string(),
                                                    pre_balances.clone(),
                                                    post_balances.clone(),
                                                    inner_td.fee_account.clone()
                                                ),
                                                quote_amount: get_amt(
                                                    &inner_td.second_swap_vault_b.clone().unwrap(),
                                                    inner_idx as u32,
                                                    &inner_instructions,
                                                    &accounts,
                                                    &post_token_balances,
                                                    "".to_string(),
                                                    pre_balances.clone(),
                                                    post_balances.clone(),
                                                    inner_td.fee_account.clone()
                                                ),
                                                base_vault: inner_td.second_swap_vault_a
                                                    .clone()
                                                    .unwrap(),
                                                quote_vault: inner_td.second_swap_vault_b
                                                    .clone()
                                                    .unwrap(),
                                                is_inner_instruction: true,
                                                instruction_index: idx as u32,
                                                instruction_type: inner_td_name.clone(),
                                                inner_instruxtion_index: inner_idx as u32,
                                                outer_program: program.to_string(),
                                                inner_program: inner_td_dapp_address.clone(),
                                                txn_fee_lamports: meta.fee,
                                                signer_lamports_change: get_signer_balance_change(
                                                    &pre_balances,
                                                    &post_balances
                                                ),
                                            });
                                        }
                                    }
                                })
                        });
                }
            }
        }
    }

    log::info!("{:#?}", slot);
    Ok(Output { data })
}

pub fn resolved_accounts(transaction: &ConfirmedTransaction) -> Vec<&Vec<u8>> {
    let meta = transaction.meta.as_ref().unwrap();
    let mut accounts = vec![];

    transaction.transaction
        .as_ref()
        .unwrap()
        .message.as_ref()
        .unwrap()
        .account_keys.iter()
        .for_each(|addr| {
            accounts.push(addr);
        });
    meta.loaded_writable_addresses.iter().for_each(|addr| {
        accounts.push(addr);
    });
    meta.loaded_readonly_addresses.iter().for_each(|addr| {
        accounts.push(addr);
    });

    accounts
}

pub fn resolved_accounts_as_strings(transaction: &ConfirmedTransaction) -> Vec<String> {
    let accounts = resolved_accounts(transaction);

    let mut resolved_accounts = vec![];

    accounts.iter().for_each(|addr| resolved_accounts.push(bs58::encode(addr).into_string()));

    resolved_accounts
}

fn get_trade_instruction(
    dapp_address: &String,
    instruction_data: Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
    pre_token_balances: &Vec<TokenBalance>,
    post_token_balances: &Vec<TokenBalance>,
    outer_program: &String,
    is_inner: bool,
    inner_instructions: &Vec<InnerInstructions>,
    input_inner_idx: u32
) -> Option<trade_instruction::TradeInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);

    let mut result = None;
    match dapp_address.as_str() {
        "5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Y65kdnB1Uev3h" => {
            result = dapps::raydium_amm::parse_trade_instruction(instruction_data, input_accounts);
        }
        "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK" => {
            result = dapps::raydium_clmm::parse_trade_instruction(instruction_data, input_accounts);
        }
        "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG" => {
            result = dapps::meteora_damm_v2::parse_trade_instruction(
                instruction_data,
                input_accounts
            );
        }
        "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP" => {
            result = dapps::orca_swap_v2::parse_trade_instruction(instruction_data, input_accounts);
        }
        "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN" => {
            result = dapps::meteora_dynamic_bonding_curve::parse_trade_instruction(
                instruction_data,
                input_accounts
            );
        }
        "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1" => {
            result = dapps::orca_swap::parse_trade_instruction(instruction_data, input_accounts);
        }
        "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB" => {
            result = dapps::meteora_damm_v1::parse_trade_instruction(
                instruction_data,
                input_accounts
            );
        }
        "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj" => {
            result = dapps::raydium_launch_lab::parse_trade_instruction(
                instruction_data,
                input_accounts
            );
        }
        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo" => {
            result = dapps::meteora_dlmm::parse_trade_instruction(instruction_data, input_accounts);
        }
        "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C" => {
            result = dapps::raydium_cpmm::parse_trade_instruction(instruction_data, input_accounts);
        }
        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => {
            result = dapps::pumpfun::parse_trade_instruction(instruction_data, input_accounts);
        }
        "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA" => {
            result = dapps::pumpswap::parse_trade_instruction(instruction_data, input_accounts);
        }
        _ => {}
    }

    return result;
}

fn get_signer_balance_change(pre_balances: &Vec<u64>, post_balances: &Vec<u64>) -> i64 {
    return (post_balances[0] - pre_balances[0]) as i64;
}

fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}

fn filter_inner_instructions(
    meta_inner_instructions: &Vec<InnerInstructions>,
    idx: u32
) -> Vec<InnerInstructions> {
    let mut inner_instructions: Vec<InnerInstructions> = vec![];
    let mut iterator = meta_inner_instructions.iter();
    while let Some(inner_inst) = iterator.next() {
        if inner_inst.index == (idx as u32) {
            inner_instructions.push(inner_inst.clone());
        }
    }
    return inner_instructions;
}
