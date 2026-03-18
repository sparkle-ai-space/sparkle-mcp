pub mod acp_component;
pub mod auto_checkpoint;
pub mod context_loader;
pub mod database;
pub mod embodiment;
pub mod prompts;
pub mod server;
pub mod sparkle_loader;
pub mod sparkle_paths;
pub mod tools;
pub mod types;

pub use acp_component::SparkleComponent;
pub use embodiment::generate_embodiment_content;
pub use server::SparkleServer;
