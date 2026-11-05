#![cfg(feature = "realtime")]

use common::contracts::realtime_internal_v1::{RealtimeCommandEnvelope, realtime_command_envelope::Command};
use common::contracts::realtime_v1::{
    MediaAttachment, MediaResourceProjection, MessageMetadata, PollMetadata, WritingCommand, message_metadata::Kind,
};
use prost::Message;
use serde_json::json;
use uuid::Uuid;

#[test]
fn command_constructor_keeps_the_existing_nats_bytes() {
    let actor = Uuid::from_bytes(std::array::from_fn(|i| i as u8));
    let room = (16_u8..32).collect();
    let envelope =
        RealtimeCommandEnvelope::new("cmd-1".into(), actor, Command::Writing(WritingCommand { room_id: room }));
    let golden = [
        0x08, 0x01, 0x12, 0x05, b'c', b'm', b'd', b'-', b'1', 0x1a, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x82, 0x01, 0x12, 0x0a, 0x10, 0x10, 0x11, 0x12, 0x13,
        0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    ];
    assert_eq!(envelope.encode_to_vec(), golden);
    assert_eq!(RealtimeCommandEnvelope::decode(golden.as_slice()).unwrap(), envelope);
}

#[test]
fn media_projection_preserves_input_without_output_urls() {
    let id = Uuid::from_bytes([7; 16]);
    let value = MediaAttachment {
        id: id.as_bytes().to_vec(),
        purpose: "voice_message".into(),
        name: Some("voice.ogg".into()),
        mime_type: Some("audio/ogg".into()),
        size: Some(u64::MAX),
        duration_ms: Some(1500),
        waveform: vec![0.25, 0.5],
        revision: Some("blake3:source".into()),
        original: Some(MediaResourceProjection { read_url: "private-original".into(), ..Default::default() }),
        display: Some(MediaResourceProjection { read_url: "private-display".into(), ..Default::default() }),
    }
    .into_command_value();
    assert_eq!(
        value,
        json!({
            "id": id,
            "purpose": "voice_message",
            "name": "voice.ogg",
            "mime_type": "audio/ogg",
            "size": u64::MAX,
            "duration_ms": 1500,
            "waveform": [0.25, 0.5],
            "revision": "blake3:source",
        })
    );
}

#[test]
fn absent_media_fields_are_omitted_and_invalid_id_is_not_projected() {
    for id in [vec![], vec![1; 15], vec![1; 17]] {
        let value = MediaAttachment { id, purpose: "attachment".into(), ..Default::default() }.into_command_value();
        assert_eq!(value, json!({"purpose": "attachment"}));
    }
}

#[test]
fn poll_projection_preserves_options_and_false_flags() {
    let value = MessageMetadata {
        kind: Some(Kind::Poll(PollMetadata {
            version: 1,
            question: "Куда?".into(),
            options: vec!["Лево".into(), "Право".into()],
            anonymous: false,
            multiple: false,
            allow_change: true,
        })),
    }
    .into_command_value();
    assert_eq!(
        value,
        json!({
            "kind": "poll", "version": 1, "question": "Куда?", "options": ["Лево", "Право"],
            "anonymous": false, "multiple": false, "allow_change": true,
        })
    );
    assert_eq!(MessageMetadata::default().into_command_value(), json!({}));
}
