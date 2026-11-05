//! Технические преобразования общих realtime-схем; права проверяют приложения.
use crate::contracts::realtime_internal_v1::{RealtimeCommandEnvelope, realtime_command_envelope::Command};
use crate::contracts::realtime_v1::{MediaAttachment, MessageMetadata, message_metadata::Kind};
use serde_json::{Map, Value, json};
use uuid::Uuid;

impl RealtimeCommandEnvelope {
    pub fn new(command_id: String, actor_id: Uuid, command: Command) -> Self {
        Self { protocol_version: 1, command_id, actor_id: actor_id.as_bytes().to_vec(), command: Some(command) }
    }
}

impl MediaAttachment {
    /// Прикладная проекция входного вложения, без выходных read URL.
    pub fn into_command_value(self) -> Value {
        let mut object = Map::new();
        if let Ok(id) = Uuid::from_slice(&self.id) {
            object.insert("id".into(), json!(id));
        }
        object.insert("purpose".into(), json!(self.purpose));
        for (key, value) in [("name", self.name), ("mime_type", self.mime_type), ("revision", self.revision)] {
            if let Some(value) = value {
                object.insert(key.into(), Value::String(value));
            }
        }
        for (key, value) in [("size", self.size), ("duration_ms", self.duration_ms)] {
            if let Some(value) = value {
                object.insert(key.into(), value.into());
            }
        }
        if !self.waveform.is_empty() {
            object.insert("waveform".into(), json!(self.waveform));
        }
        Value::Object(object)
    }
}

impl MessageMetadata {
    pub fn into_command_value(self) -> Value {
        match self.kind {
            Some(Kind::Poll(poll)) => json!({
                "kind": "poll",
                "version": poll.version,
                "question": poll.question,
                "options": poll.options,
                "anonymous": poll.anonymous,
                "multiple": poll.multiple,
                "allow_change": poll.allow_change,
            }),
            None => json!({}),
        }
    }
}
