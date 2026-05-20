pub mod builder;
pub mod library;

pub use builder::{build_provider_prompt, PromptMetadata};
pub use library::{
    write_prompts_to_dir, CLAUDE_PLANNER_AGENT, CODEX_SCOPED_EXECUTOR, CONTEXT_PACKET_AGENT,
    CURSOR_COMPOSER_EXECUTION_AGENT, OLLAMA_COMPRESSION_AGENT, PROJECT_MEMORY_AGENT, REVIEW_AGENT,
    ROUTER_AGENT,
};
