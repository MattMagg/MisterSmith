use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::mpsc::error::TryRecvError;
use uuid::Uuid;

use crate::errors::AgentSystemError;
use crate::orchestrator::LlmSupervision;
use mister_smith_llm::{
    CompletionRequest, CompletionResponse, ContentBlock, DualStreamActor, DualStreamConfig,
    DualStreamHandle, ModelEvent, ModelProvider, ModelRouter, StopReason, ToolCall, Usage,
};

pub(crate) async fn complete_with_optional_supervision(
    router: &Arc<ModelRouter>,
    request: CompletionRequest,
    supervision: Option<&LlmSupervision>,
) -> Result<CompletionResponse, AgentSystemError> {
    if let Some(supervision) = supervision {
        return stream_with_supervision(router.as_ref(), request, supervision).await;
    }

    let (response, _routing) = router.route_completion(request).await?;
    Ok(response)
}

async fn stream_with_supervision(
    router: &ModelRouter,
    request: CompletionRequest,
    supervision: &LlmSupervision,
) -> Result<CompletionResponse, AgentSystemError> {
    supervision.request_started(router.model_id()).await?;

    let mut stream = router.stream(request);
    let request_id = Uuid::new_v4().to_string();
    let (mut actor, mut handle) = DualStreamActor::new(DualStreamConfig::default());
    let mut response = StreamedCompletion::default();

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(chunk) => chunk,
            Err(error) => {
                supervision.completion_failed(&error).await?;
                return Err(error.into());
            }
        };

        actor
            .process_chunk(chunk, router.model_id(), &request_id)
            .await;
        drain_stream_events(&mut handle, supervision, &mut response).await?;
    }

    actor.finish().await;
    drain_stream_events(&mut handle, supervision, &mut response).await?;

    let response = response.into_completion_response(router.model_id().to_string());
    supervision.completion_succeeded(&response).await?;
    Ok(response)
}

async fn drain_stream_events(
    handle: &mut DualStreamHandle,
    supervision: &LlmSupervision,
    response: &mut StreamedCompletion,
) -> Result<(), AgentSystemError> {
    drain_receiver(&mut handle.semantic_rx, supervision, response).await?;
    drain_receiver(&mut handle.ui_rx, supervision, response).await?;
    Ok(())
}

async fn drain_receiver(
    receiver: &mut tokio::sync::mpsc::Receiver<ModelEvent>,
    supervision: &LlmSupervision,
    response: &mut StreamedCompletion,
) -> Result<(), AgentSystemError> {
    loop {
        match receiver.try_recv() {
            Ok(event) => {
                response.observe(&event);
                let _ = supervision.observe_model_event(&event).await?;
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => return Ok(()),
        }
    }
}

#[derive(Debug, Default)]
struct StreamedCompletion {
    text: String,
    tool_calls: Vec<ToolCall>,
    usage: Usage,
    stop_reason: StopReason,
}

impl StreamedCompletion {
    fn observe(&mut self, event: &ModelEvent) {
        match event {
            ModelEvent::TextDelta { text } => self.text.push_str(text),
            ModelEvent::ToolCallCompleted {
                call_id,
                name,
                input,
            } => {
                self.tool_calls.push(ToolCall {
                    call_id: call_id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                });
            }
            ModelEvent::UsageUpdate { usage } => {
                self.usage = *usage;
            }
            ModelEvent::StreamCompleted { usage, stop_reason } => {
                self.usage = *usage;
                self.stop_reason = stop_reason.clone();
            }
            _ => {}
        }
    }

    fn into_completion_response(self, model_id: String) -> CompletionResponse {
        let mut content = Vec::new();
        if !self.text.is_empty() {
            content.push(ContentBlock::Text { text: self.text });
        }
        content.extend(self.tool_calls.iter().map(|call| ContentBlock::ToolUse {
            call_id: call.call_id.clone(),
            name: call.name.clone(),
            input: call.input.clone(),
        }));

        CompletionResponse {
            content,
            model_id,
            usage: self.usage,
            stop_reason: self.stop_reason,
            tool_calls: self.tool_calls,
        }
    }
}
