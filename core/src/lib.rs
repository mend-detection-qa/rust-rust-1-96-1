// probe-core: minimal library stub.
// Imports keep Cargo from stripping the dependencies at check time.
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;

/// A trivial serializable config to exercise serde + thiserror.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub greeting: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("template error: {0}")]
    Template(#[from] tinytemplate::error::Error),
}

static LOGGER_INIT: Lazy<()> = Lazy::new(|| {
    let _ = log::logger();
});

pub fn render(template: &str, ctx: &Config) -> Result<String, CoreError> {
    let _ = *LOGGER_INIT;
    let mut tt = TinyTemplate::new();
    tt.add_template("t", template)?;
    Ok(tt.render("t", ctx)?)
}
