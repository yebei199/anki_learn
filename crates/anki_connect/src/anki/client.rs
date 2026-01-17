use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Default Anki-Connect endpoint URL
const DEFAULT_ANKI_CONNECT_URL: &str =
    "http://localhost:8765";

/// Anki-Connect request structure following JSON-RPC 2.0 specification
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnkiRequest<T> {
    /// The action to perform (e.g., "version", "addNote", "getDeckNames")
    action: String,
    /// Version of the API (currently always 6)
    version: u8,
    /// Action-specific parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

impl<T> AnkiRequest<T> {
    /// Creates a new Anki-Connect request
    fn new(
        action: &str,
        version: u8,
        params: Option<T>,
    ) -> Self {
        Self {
            action: action.to_string(),
            version,
            params,
        }
    }
}

/// Anki-Connect response structure
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AnkiResponse<T> {
    /// Successful response containing the result
    Success { result: T },
    /// Error response containing error details
    Error {
        error: String,
        #[serde(default)]
        detail: Option<String>,
    },
}

/// Represents a single note field (key-value pair)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: String,
}

/// Represents a note to be added to Anki
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// Unique identifier for the note type (model)
    pub model_name: String,
    /// Deck name where the note should be added
    pub deck_name: String,
    /// Fields of the note (e.g., Front, Back)
    pub fields: std::collections::HashMap<String, String>,
    /// Tags for the note
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional: Audio files to add with the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Vec<NoteAudio>>,
    /// Optional: Pictures to add with the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<Vec<NotePicture>>,
    /// Optional: Video files to add with the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Vec<NoteVideo>>,
    /// Optional: Check for duplicates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<NoteOptions>,
}

/// Audio file attached to a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteAudio {
    /// Path to the audio file
    pub path: String,
    /// Filename to use in Anki
    pub filename: String,
    /// Field name where audio should be embedded
    pub fields: Vec<String>,
    /// Optional: SHA-256 hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Picture attached to a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotePicture {
    /// Path to the picture file
    pub path: String,
    /// Filename to use in Anki
    pub filename: String,
    /// Field name where picture should be embedded
    pub fields: Vec<String>,
    /// Optional: SHA-256 hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Video file attached to a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteVideo {
    /// Path to the video file
    pub path: String,
    /// Filename to use in Anki
    pub filename: String,
    /// Field name where video should be embedded
    pub fields: Vec<String>,
    /// Optional: SHA-256 hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Options for note creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteOptions {
    /// Whether to check for duplicates
    #[serde(rename = "allowDuplicate")]
    pub allow_duplicate: bool,
    /// Scope of duplicate check
    #[serde(rename = "duplicateScope")]
    pub duplicate_scope: Option<String>,
}

/// Parameters for adding notes in bulk
#[derive(Debug, Clone, Serialize)]
pub struct AddNotesParams {
    /// List of notes to add
    pub notes: Vec<Note>,
}

/// Parameters for adding a single note
#[derive(Debug, Clone, Serialize)]
pub struct AddNoteParams {
    /// Note to add
    pub note: Note,
}

/// Parameters for getting deck names
#[derive(Debug, Clone, Serialize)]
pub struct GetDeckNamesParams {
    /// Whether to include cards
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<bool>,
}

/// Parameters for getting model names
#[derive(Debug, Clone, Serialize)]
pub struct GetModelNamesParams {}

/// Parameters for getting field names
#[derive(Debug, Clone, Serialize)]
pub struct GetModelFieldNamesParams {
    /// Model name
    #[serde(rename = "modelName")]
    pub model_name: String,
}

/// Information about a note in Anki
#[derive(Debug, Clone, Deserialize)]
pub struct NoteInfo {
    /// Note ID
    pub note_id: u64,
    /// Tags for the note
    #[serde(default)]
    pub tags: Vec<String>,
    /// Fields of the note
    pub fields:
        std::collections::HashMap<String, NoteFieldValue>,
    /// Model name
    #[serde(rename = "modelName")]
    pub model_name: String,
    /// Cards associated with this note
    pub cards: Vec<u64>,
}

/// Value of a note field with ordering info
#[derive(Debug, Clone, Deserialize)]
pub struct NoteFieldValue {
    /// Field value
    pub value: String,
    /// Field order
    pub order: u32,
}

/// Parameters for finding notes
#[derive(Debug, Clone, Serialize)]
pub struct FindNotesParams {
    /// Search query
    pub query: String,
}

/// Parameters for getting notes info
#[derive(Debug, Clone, Serialize)]
pub struct NotesInfoParams {
    /// List of note IDs
    pub notes: Vec<u64>,
}

/// Parameters for updating note fields
#[derive(Debug, Clone, Serialize)]
pub struct UpdateNoteFieldsParams {
    /// Note ID
    pub note: u64,
    /// Fields to update
    pub fields: std::collections::HashMap<String, String>,
    /// Audio files (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Vec<NoteAudio>>,
}

/// Information about a card in Anki
#[derive(Debug, Clone, Deserialize)]
pub struct CardInfo {
    /// Card ID
    pub card_id: u64,
    /// Note ID
    pub note_id: u64,
    /// Deck name
    #[serde(rename = "deck")]
    pub deck_name: String,
    /// Model name
    #[serde(rename = "modelName")]
    pub model_name: String,
    /// Card ordinal
    pub ord: u32,
    /// Card modification time
    #[serde(rename = "mod")]
    pub modification_time: u64,
    /// Card type (0=new, 1=learning, 2=review, 3=relearning)
    #[serde(rename = "type")]
    pub card_type: u32,
    /// Card queue
    pub queue: u32,
    /// Card due time
    pub due: u64,
    /// Card interval
    pub interval: u32,
    /// Card ease factor
    pub factor: u32,
    /// Number of repetitions
    pub reps: u32,
    /// Number of lapses
    pub lapses: u32,
    /// Card left value
    pub left: u32,
    /// Original due time
    #[serde(rename = "odue")]
    pub original_due: u64,
    /// Original card queue
    #[serde(rename = "oqueue")]
    pub original_queue: u32,
    /// Card flags
    pub flags: u32,
}

/// Parameters for getting cards info
#[derive(Debug, Clone, Serialize)]
pub struct CardsInfoParams {
    /// List of card IDs
    pub cards: Vec<u64>,
}

/// Parameters for finding cards
#[derive(Debug, Clone, Serialize)]
pub struct FindCardsParams {
    /// Search query
    pub query: String,
}

/// Anki-Connect client for interacting with Anki
#[derive(Debug, Clone)]
pub struct AnkiClient {
    /// HTTP client for making requests
    client: Client,
    /// Anki-Connect endpoint URL
    url: String,
    /// API version
    version: u8,
}

impl Default for AnkiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AnkiClient {
    /// Creates a new AnkiClient with default settings
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            url: DEFAULT_ANKI_CONNECT_URL.to_string(),
            version: 6,
        }
    }

    /// Creates a new AnkiClient with a custom endpoint URL
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            url: url.into(),
            version: 6,
        }
    }

    /// Creates a new AnkiClient with custom HTTP client
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            url: DEFAULT_ANKI_CONNECT_URL.to_string(),
            version: 6,
        }
    }

    /// Invokes an Anki-Connect action with the given parameters
    async fn invoke<T, R>(
        &self,
        action: &str,
        params: Option<T>,
    ) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request =
            AnkiRequest::new(action, self.version, params);
        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .context(
                "Failed to send request to Anki-Connect",
            )?;

        let text = response.text().await.context(
            "Failed to read response from Anki-Connect",
        )?;

        let anki_response: AnkiResponse<R> =
            serde_json::from_str(&text).context(
                "Failed to parse Anki-Connect response",
            )?;

        match anki_response {
            AnkiResponse::Success { result } => Ok(result),
            AnkiResponse::Error { error, detail } => {
                let error_msg = if let Some(detail) = detail
                {
                    format!("{}: {}", error, detail)
                } else {
                    error
                };
                Err(anyhow::anyhow!(
                    "Anki-Connect error: {}",
                    error_msg
                ))
            }
        }
    }

    /// Gets the Anki-Connect API version
    pub async fn version(&self) -> Result<u32> {
        self.invoke::<(), u32>("version", None).await
    }

    /// Gets the names of all decks in the collection
    pub async fn get_deck_names(
        &self,
        cards: Option<bool>,
    ) -> Result<Vec<String>> {
        let params = if cards.is_some() {
            Some(GetDeckNamesParams { cards })
        } else {
            None
        };
        self.invoke("getDeckNames", params).await
    }

    /// Gets the names of all models in the collection
    pub async fn get_model_names(
        &self,
    ) -> Result<Vec<String>> {
        self.invoke(
            "getModelNames",
            None::<GetModelNamesParams>,
        )
        .await
    }

    /// Gets the field names for a specific model
    pub async fn get_model_field_names(
        &self,
        model_name: &str,
    ) -> Result<Vec<String>> {
        let params = GetModelFieldNamesParams {
            model_name: model_name.to_string(),
        };
        self.invoke("getModelFieldNames", Some(params))
            .await
    }

    /// Adds a single note to Anki
    pub async fn add_note(
        &self,
        note: Note,
    ) -> Result<u64> {
        let params = AddNoteParams { note };
        self.invoke("addNote", Some(params)).await
    }

    /// Adds multiple notes to Anki in a single request
    pub async fn add_notes(
        &self,
        notes: Vec<Note>,
    ) -> Result<Vec<Option<u64>>> {
        let params = AddNotesParams { notes };
        self.invoke("addNotes", Some(params)).await
    }

    /// Finds notes matching the given query
    pub async fn find_notes(
        &self,
        query: &str,
    ) -> Result<Vec<u64>> {
        let params = FindNotesParams {
            query: query.to_string(),
        };
        self.invoke("findNotes", Some(params)).await
    }

    /// Gets detailed information about notes
    pub async fn notes_info(
        &self,
        note_ids: Vec<u64>,
    ) -> Result<Vec<NoteInfo>> {
        let params = NotesInfoParams { notes: note_ids };
        self.invoke("notesInfo", Some(params)).await
    }

    /// Updates fields of an existing note
    pub async fn update_note_fields(
        &self,
        note_id: u64,
        fields: std::collections::HashMap<String, String>,
        audio: Option<Vec<NoteAudio>>,
    ) -> Result<()> {
        let params = UpdateNoteFieldsParams {
            note: note_id,
            fields,
            audio,
        };
        let result: bool = self
            .invoke("updateNoteFields", Some(params))
            .await?;
        if result {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to update note fields"
            ))
        }
    }

    /// Gets detailed information about cards
    pub async fn cards_info(
        &self,
        card_ids: Vec<u64>,
    ) -> Result<Vec<CardInfo>> {
        let params = CardsInfoParams { cards: card_ids };
        self.invoke("cardsInfo", Some(params)).await
    }

    /// Finds cards matching the given query
    pub async fn find_cards(
        &self,
        query: &str,
    ) -> Result<Vec<u64>> {
        let params = FindCardsParams {
            query: query.to_string(),
        };
        self.invoke("findCards", Some(params)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anki_request_serialization() {
        let request =
            AnkiRequest::new("version", 6, None::<()>);
        let json = serde_json::to_string(&request)
            .expect("Failed to serialize request");
        assert_eq!(
            json,
            r#"{"action":"version","version":6}"#
        );
    }

    #[test]
    fn test_anki_request_with_params_serialization() {
        let params =
            GetDeckNamesParams { cards: Some(true) };
        let request = AnkiRequest::new(
            "getDeckNames",
            6,
            Some(params),
        );
        let json = serde_json::to_string(&request)
            .expect("Failed to serialize request");
        assert!(json.contains(r#"action":"getDeckNames""#));
        assert!(json.contains(r#"version":6"#));
        assert!(json.contains(r#"cards":true"#));
    }

    #[test]
    fn test_anki_response_success_deserialization() {
        let json = r#"{"result":12345}"#;
        let response: AnkiResponse<u64> =
            serde_json::from_str(json)
                .expect("Failed to deserialize response");
        match response {
            AnkiResponse::Success { result } => {
                assert_eq!(result, 12345)
            }
            AnkiResponse::Error { .. } => {
                panic!("Expected success response")
            }
        }
    }

    #[test]
    fn test_anki_response_error_deserialization() {
        let json = r#"{"error":"Test error","detail":"Test detail"}"#;
        let response: AnkiResponse<u64> =
            serde_json::from_str(json)
                .expect("Failed to deserialize response");
        match response {
            AnkiResponse::Success { .. } => {
                panic!("Expected error response")
            }
            AnkiResponse::Error { error, detail } => {
                assert_eq!(error, "Test error");
                assert_eq!(
                    detail,
                    Some("Test detail".to_string())
                );
            }
        }
    }

    #[test]
    fn test_note_serialization() {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "Front".to_string(),
            "Question".to_string(),
        );
        fields.insert(
            "Back".to_string(),
            "Answer".to_string(),
        );

        let note = Note {
            model_name: "Basic".to_string(),
            deck_name: "Default".to_string(),
            fields,
            tags: vec!["test".to_string()],
            audio: None,
            picture: None,
            video: None,
            options: None,
        };

        let json = serde_json::to_string(&note)
            .expect("Failed to serialize note");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert_eq!(parsed["modelName"], "Basic");
        assert_eq!(parsed["deckName"], "Default");
        assert_eq!(parsed["tags"][0], "test");
    }

    #[test]
    fn test_note_with_audio_serialization() {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "Front".to_string(),
            "Question".to_string(),
        );
        fields.insert(
            "Back".to_string(),
            "Answer".to_string(),
        );

        let audio = NoteAudio {
            path: "/path/to/audio.mp3".to_string(),
            filename: "audio.mp3".to_string(),
            fields: vec!["Back".to_string()],
            hash: Some("abc123".to_string()),
        };

        let note = Note {
            model_name: "Basic".to_string(),
            deck_name: "Default".to_string(),
            fields,
            tags: vec![],
            audio: Some(vec![audio]),
            picture: None,
            video: None,
            options: None,
        };

        let json = serde_json::to_string(&note)
            .expect("Failed to serialize note");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert!(parsed["audio"].is_array());
        assert_eq!(
            parsed["audio"][0]["filename"],
            "audio.mp3"
        );
    }

    #[test]
    fn test_client_creation() {
        let client = AnkiClient::new();
        assert_eq!(client.url, DEFAULT_ANKI_CONNECT_URL);
        assert_eq!(client.version, 6);
    }

    #[test]
    fn test_client_with_custom_url() {
        let client =
            AnkiClient::with_url("http://custom:8765");
        assert_eq!(client.url, "http://custom:8765");
    }

    #[test]
    fn test_add_notes_params_serialization() {
        let mut fields = std::collections::HashMap::new();
        fields
            .insert("Front".to_string(), "Q1".to_string());
        fields.insert("Back".to_string(), "A1".to_string());

        let note = Note {
            model_name: "Basic".to_string(),
            deck_name: "Default".to_string(),
            fields,
            tags: vec![],
            audio: None,
            picture: None,
            video: None,
            options: None,
        };

        let params = AddNotesParams { notes: vec![note] };
        let json = serde_json::to_string(&params)
            .expect("Failed to serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert!(parsed["notes"].is_array());
        assert_eq!(
            parsed["notes"].as_array().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_update_note_fields_params_serialization() {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "Front".to_string(),
            "New Question".to_string(),
        );

        let params = UpdateNoteFieldsParams {
            note: 12345,
            fields,
            audio: None,
        };

        let json = serde_json::to_string(&params)
            .expect("Failed to serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert_eq!(parsed["note"], 12345);
        assert_eq!(
            parsed["fields"]["Front"],
            "New Question"
        );
    }

    #[test]
    fn test_find_notes_params_serialization() {
        let params = FindNotesParams {
            query: "deck:Default".to_string(),
        };

        let json = serde_json::to_string(&params)
            .expect("Failed to serialize");
        assert_eq!(json, r#"{"query":"deck:Default"}"#);
    }

    #[test]
    fn test_notes_info_params_serialization() {
        let params = NotesInfoParams {
            notes: vec![123, 456, 789],
        };

        let json = serde_json::to_string(&params)
            .expect("Failed to serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert_eq!(parsed["notes"][0], 123);
        assert_eq!(parsed["notes"][1], 456);
        assert_eq!(parsed["notes"][2], 789);
    }

    #[test]
    fn test_cards_info_params_serialization() {
        let params = CardsInfoParams {
            cards: vec![111, 222, 333],
        };

        let json = serde_json::to_string(&params)
            .expect("Failed to serialize");
        let parsed: serde_json::Value =
            serde_json::from_str(&json)
                .expect("Failed to parse JSON");
        assert_eq!(parsed["cards"][0], 111);
        assert_eq!(parsed["cards"][1], 222);
        assert_eq!(parsed["cards"][2], 333);
    }
}
