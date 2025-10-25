pub mod schemas;
pub mod ya;

// Re-export key rig components that users of this crate will need
pub mod prelude
{
  pub use crate::ya;
  pub use rig;
  pub use rig::agent::AgentBuilder;
  pub use rig::client::builder::*;
  pub use rig::client::completion::CompletionModelHandle;
  pub use rig::client::*;
  pub use rig::completion::Prompt;
  pub use rig::message::*;
}
