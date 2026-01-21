use anyhow::{anyhow, Result};

pub fn validate_rpc_url(label: &str, value: &str) -> Result<()> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(())
    } else {
        Err(anyhow!("{label} must start with http:// or https://"))
    }
}

pub fn validate_nonzero(label: &str, value: u64) -> Result<()> {
    if value == 0 {
        Err(anyhow!("{label} must be greater than zero"))
    } else {
        Ok(())
    }
}
