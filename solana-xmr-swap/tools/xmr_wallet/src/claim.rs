use crate::types::{Result, XmrWalletError};

#[derive(Debug, Clone)]
pub struct ClaimPlan {
    pub description: String,
}

pub async fn build_claim_plan(_swap_id: &str) -> Result<ClaimPlan> {
    Err(XmrWalletError::Unsupported(
        "claim plan builder not implemented yet",
    ))
}
