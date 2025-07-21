#[derive(Debug)]
pub struct TradeInstruction {
    pub dapp_address: String,
    pub name: String,
    pub amm: String,
    pub vault_a: String,
    pub vault_b: String,
    pub second_swap_amm: Option<String>,
    pub second_swap_vault_a: Option<String>,
    pub second_swap_vault_b: Option<String>,
    pub fee_account: Option<String>,
}

impl Default for TradeInstruction {
    fn default() -> Self {
        TradeInstruction {
            dapp_address: String::new(),
            name: String::new(),
            amm: String::new(),
            vault_a: String::new(),
            vault_b: String::new(),
            second_swap_amm: None,
            second_swap_vault_a: None,
            second_swap_vault_b: None,
            fee_account: None,
        }
    }
}
