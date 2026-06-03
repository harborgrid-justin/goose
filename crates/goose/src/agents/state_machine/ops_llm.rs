use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use rmcp::model::Role;

use crate::agents::state_machine::operation::{Emitter, Operation, TurnOutcome};
use crate::agents::AgentEvent;
use crate::conversation::Conversation;
use crate::providers::base::Provider;
use crate::session::Session;

/// Calls the LLM when the last message in the conversation is from the user.
pub struct LlmOperation {
    provider: Arc<dyn Provider>,
    system_prompt: String,
}

impl LlmOperation {
    pub fn new(provider: Arc<dyn Provider>, system_prompt: String) -> Self {
        Self {
            provider,
            system_prompt,
        }
    }
}

#[async_trait]
impl Operation for LlmOperation {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn applies(&self, session: &Session) -> bool {
        matches!(
            session
                .conversation
                .as_ref()
                .and_then(|c| c.messages().last())
                .map(|m| &m.role),
            Some(Role::User)
        )
    }

    async fn run(&self, session: &Session, emit: Emitter) -> Result<TurnOutcome> {
        let conversation = session
            .conversation
            .as_ref()
            .ok_or_else(|| anyhow!("LlmOperation::run with no conversation"))?;

        let model_config = self.provider.get_model_config();
        let messages_for_provider: Vec<_> = conversation
            .messages()
            .iter()
            .filter(|m| m.is_agent_visible())
            .map(|m| m.agent_visible_content())
            .collect();

        let mut stream = self
            .provider
            .stream(
                &model_config,
                &session.id,
                &self.system_prompt,
                &messages_for_provider,
                &[],
            )
            .await?;

        // Conversation::push handles merge logic — coalescing text, merging
        // thinking blocks by signature, deduping by message id, forwarding
        // inference metadata to the right prior message.
        let mut accumulator = Conversation::empty();
        while let Some(result) = stream.next().await {
            let (msg_opt, _usage_opt) = result?;
            if let Some(chunk) = msg_opt {
                emit.emit(AgentEvent::Message(chunk.clone())).await;
                accumulator.push(chunk);
            }
        }

        Ok(TurnOutcome::AppendMessages(
            accumulator.into_iter().collect(),
        ))
    }
}
