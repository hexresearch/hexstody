pub const WITHDRAWAL_CONFIRM_URI: &str = "/confirm";
pub const WITHDRAWAL_REJECT_URI: &str = "/reject";

#[derive(Debug, Clone)]
pub struct ConfirmationsConfig {
    // Number of confirmations from operators required for funds withdrawal above the limit
    pub withdraw: i16,
    // Number of confirmations from operators required for change withdrawal limits
    pub change_limit: i16,
    // Number of confirmations from operators required for exchange
    pub exchange: i16,
}

impl ConfirmationsConfig {
    // Returns maximum value among fields
    pub fn max(&self) -> i16 {
        let items = [self.withdraw, self.change_limit, self.exchange];
        items
            .iter()
            .copied()
            .reduce(|accum, item| if accum >= item { accum } else { item })
            .unwrap()
    }
}

// Should be the same as hexstody-db::state::CONFIRMATIONS_CONFIG
pub const CONFIRMATIONS_CONFIG: ConfirmationsConfig = ConfirmationsConfig {
    withdraw: 2,
    change_limit: 2,
    exchange: 1,
};
