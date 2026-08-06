use axum::response::{Html, IntoResponse};
use axum::Json;
use serde_json::{json, Value};

const DOCS_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Metocast API Reference</title>
  </head>
  <body>
    <script id="api-reference" data-url="/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>"#;

pub async fn serve_spec() -> impl IntoResponse {
    Json(spec())
}

pub async fn serve_docs() -> Html<&'static str> {
    Html(DOCS_HTML)
}

pub fn spec() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Metocast API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "REST API and WebSocket interface for the Metocast desktop application.\n\n## Authentication\n\nAll `/api/*` endpoints require a **Bearer token** in the `Authorization` header:\n```\nAuthorization: Bearer <token>\n```\nThe token is displayed in the app's *Connection Guide* screen. It rotates on every server restart.\n\n## WebSocket — real-time push stream\n\n> **Note:** WebSocket is not an HTTP operation and cannot be tested from this page. Use a WebSocket client (e.g. [Hoppscotch](https://hoppscotch.io), [websocat](https://github.com/vi/websocat), or Bruno's socket type).\n\n**Endpoint:** `ws://<host>/ws?token=<token>`\n\nAuthentication uses the same bearer token passed as a **query parameter** (headers are not available during the WebSocket handshake).\n\n### Initial messages (sent immediately on connect)\n\n```json\n{ \"type\": \"connected\", \"serverId\": \"<uuid>\" }\n{ \"type\": \"connector.status\", \"connector\": \"obs\",  \"status\": { \"type\": \"connected\" } }\n{ \"type\": \"connector.status\", \"connector\": \"vmix\", \"status\": { \"type\": \"disconnected\" } }\n```\n\n### Push messages (broadcast on change)\n\n| `type` | Trigger | Schema |\n|---|---|---|\n| `connector.status` | OBS or VMix connection state changes | `WsConnectorStatusMessage` |\n| `event.changed` | Event created, updated, or deleted | `WsEventChangedMessage` |\n| `recording.changed` | Recording created or updated | `WsRecordingChangedMessage` |\n\n```json\n{ \"type\": \"connector.status\", \"connector\": \"obs\", \"status\": { \"type\": \"error\", \"message\": \"connection refused\" } }\n{ \"type\": \"event.changed\",     \"data\": { \"operation\": \"INSERT\", \"record\": { ...Event } } }\n{ \"type\": \"recording.changed\", \"data\": { \"operation\": \"UPDATE\", \"record\": { ...Recording } } }\n```\n\nFull payload definitions are in the `Ws*Message` schemas below."
        },
        "servers": [
            {
                "url": "/",
                "description": "Current server — replace host and port as needed (default port: 3737)"
            }
        ],
        "security": [
            { "bearerAuth": [] }
        ],
        "tags": [
            { "name": "Events",     "description": "Service events" },
            { "name": "Recordings", "description": "Video recording files linked to events" },
            { "name": "Connectors", "description": "Connector status, configuration and control" },
            { "name": "Bible",      "description": "Bible passage lookups and reference autocomplete" },
            { "name": "Presenter",  "description": "Web presenter — parse .pptx files and push slide changes to all connected browsers" },
            { "name": "WebSocket",  "description": "Real-time push stream — requires a WebSocket client, not HTTP" }
        ],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "Token shown in the app Connection Guide. Rotates on every server restart."
                }
            },
            "schemas": {
                "SlideContent": {
                    "type": "object",
                    "description": "Text content extracted from a single slide.",
                    "required": ["index", "paragraphs"],
                    "properties": {
                        "index": { "type": "integer", "minimum": 1, "description": "1-based slide number" },
                        "paragraphs": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["lines", "align", "fontSizePt"],
                                "properties": {
                                    "lines": { "type": "array", "items": { "type": "string" } },
                                    "align": { "type": "string", "enum": ["left", "center", "right", "justify"] },
                                    "fontSizePt": { "type": "number" }
                                }
                            }
                        }
                    }
                },
                "SvgSlideContent": {
                    "type": "object",
                    "description": "Self-contained SVG rendering of a single slide.",
                    "required": ["index", "svg", "widthPx", "heightPx"],
                    "properties": {
                        "index": { "type": "integer", "minimum": 1 },
                        "svg": { "type": "string", "description": "Inline SVG document. Embedded images are data URIs." },
                        "widthPx": { "type": "integer", "minimum": 1 },
                        "heightPx": { "type": "integer", "minimum": 1 }
                    }
                },
                "PresenterState": {
                    "type": "object",
                    "required": ["loaded", "filePath", "currentSlide", "totalSlides", "renderMode", "slides", "svgSlides", "muted"],
                    "properties": {
                        "loaded": { "type": "boolean" },
                        "filePath": { "type": ["string", "null"] },
                        "currentSlide": { "type": "integer", "minimum": 0 },
                        "totalSlides": { "type": "integer", "minimum": 0 },
                        "renderMode": { "type": "string", "enum": ["text", "svg"] },
                        "slides": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/SlideContent" },
                            "description": "Text slides for text mode and fallback/editing data."
                        },
                        "svgSlides": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/SvgSlideContent" },
                            "description": "SVG slides when renderMode is svg."
                        },
                        "muted": { "type": "boolean" },
                        "slideWidthEmu": { "type": "integer" },
                        "slideHeightEmu": { "type": "integer" }
                    }
                },
                "ParsedPresentation": {
                    "type": "object",
                    "description": "Structured text content extracted from a parsed .pptx file.",
                    "required": ["filePath", "totalSlides", "slides", "slideWidthEmu", "slideHeightEmu"],
                    "properties": {
                        "filePath":    { "type": "string", "example": "/Users/admin/Presentations/sunday-service.pptx" },
                        "totalSlides": { "type": "integer", "minimum": 0 },
                        "slides": {
                            "type": "array",
                            "items": { "$ref": "#/components/schemas/SlideContent" }
                        },
                        "slideWidthEmu": { "type": "integer" },
                        "slideHeightEmu": { "type": "integer" }
                    }
                },
                "BibleReference": {
                    "type": "object",
                    "required": ["type", "reference", "translation", "verses"],
                    "properties": {
                        "type":        { "type": "string", "enum": ["textus", "leckio"] },
                        "reference":   { "type": "string", "example": "John 3:16" },
                        "translation": { "type": "string", "example": "UF" },
                        "verses": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["chapter", "verse", "text"],
                                "properties": {
                                    "chapter": { "type": "integer" },
                                    "verse":   { "type": "integer" },
                                    "text":    { "type": "string" }
                                }
                            }
                        }
                    }
                },
                "Event": {
                    "type": "object",
                    "description": "Full event record including platform connections and bible references.",
                    "required": [
                        "id", "title", "dateTime", "speaker", "description",
                        "autoUploadEnabled", "connections", "bibleReferences", "createdAt", "updatedAt"
                    ],
                    "properties": {
                        "id":               { "type": "string", "format": "uuid" },
                        "title":            { "type": "string", "example": "Sunday Morning Service" },
                        "dateTime":         { "type": "string", "format": "date-time", "description": "Scheduled date and time (ISO 8601 / UTC)" },
                        "speaker":          { "type": "string", "example": "Pastor Smith" },
                        "description":      { "type": "string" },
                        "autoUploadEnabled":{ "type": "boolean" },
                        "connections":      { "type": "array", "items": { "$ref": "#/components/schemas/EventConnection" } },
                        "bibleReferences":  { "type": "array", "items": { "$ref": "#/components/schemas/BibleReference" } },
                        "createdAt":        { "type": "string", "format": "date-time" },
                        "updatedAt":        { "type": "string", "format": "date-time" }
                    }
                },
                "EventSummary": {
                    "type": "object",
                    "description": "Lightweight event entry returned by the list endpoint. Omits large text fields; adds a recording count.",
                    "required": ["id", "title", "dateTime", "speaker", "recordingCount", "createdAt", "updatedAt"],
                    "properties": {
                        "id":             { "type": "string", "format": "uuid" },
                        "title":          { "type": "string", "example": "Sunday Morning Service" },
                        "dateTime":       { "type": "string", "format": "date-time" },
                        "speaker":        { "type": "string" },
                        "recordingCount": { "type": "integer", "format": "int64", "description": "Number of recording files attached to this event" },
                        "createdAt":      { "type": "string", "format": "date-time" },
                        "updatedAt":      { "type": "string", "format": "date-time" }
                    }
                },
                "CreateEventRequest": {
                    "type": "object",
                    "description": "Request body for creating or fully replacing an event. Field names are **snake_case**.",
                    "required": ["title", "date_time"],
                    "properties": {
                        "title":             { "type": "string", "example": "Sunday Morning Service" },
                        "date_time":         { "type": "string", "format": "date-time", "description": "Scheduled date and time" },
                        "speaker":           { "type": "string", "default": "" },
                        "description":       { "type": "string", "default": "" },
                        "auto_upload_enabled": { "type": "boolean", "default": false },
                        "bible_references":  {
                            "type": "array",
                            "description": "Bible readings for the event. Each entry has a type (textus or leckio). An empty reference string removes the existing entry.",
                            "items": {
                                "type": "object",
                                "required": ["type"],
                                "properties": {
                                    "type":        { "type": "string", "enum": ["textus", "leckio"] },
                                    "reference":   { "type": "string" },
                                    "translation": { "type": "string", "default": "UF" },
                                    "verses":      { "type": "array", "items": { "$ref": "#/components/schemas/BibleReference/properties/verses/items" } }
                                }
                            }
                        },
                        "connections": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["platform"],
                                "properties": {
                                    "platform":       { "type": "string" },
                                    "privacy_status": { "type": "string" }
                                }
                            }
                        }
                    }
                },
                "Recording": {
                    "type": "object",
                    "description": "Video recording file linked to an event.",
                    "required": [
                        "id", "eventId", "filePath", "fileName", "fileSize",
                        "durationSeconds", "detectedAt", "whitelisted", "uploaded",
                        "createdAt", "updatedAt"
                    ],
                    "properties": {
                        "id":              { "type": "string", "format": "uuid" },
                        "eventId":         { "type": "string", "format": "uuid" },
                        "filePath":        { "type": "string", "description": "Absolute path on the server's filesystem" },
                        "fileName":        { "type": "string", "example": "service-2025-01-19.mp4" },
                        "fileSize":        { "type": "integer", "format": "int64", "description": "File size in bytes" },
                        "durationSeconds": { "type": "number",  "format": "double",  "description": "Duration in seconds" },
                        "detectedAt":      { "type": "string",  "format": "date-time", "description": "When the file was detected or added" },
                        "whitelisted":     { "type": "boolean", "description": "Approved for YouTube upload" },
                        "uploaded":        { "type": "boolean", "description": "Whether the file has been uploaded to YouTube" },
                        "uploadedAt":      { "type": ["string", "null"], "format": "date-time" },
                        "videoId":         { "type": ["string", "null"], "description": "YouTube video ID (set after upload)" },
                        "videoUrl":        { "type": ["string", "null"], "description": "YouTube watch URL (set after upload)" },
                        "customTitle":     { "type": ["string", "null"], "description": "Custom YouTube title; falls back to the event title when null" },
                        "createdAt":       { "type": "string", "format": "date-time" },
                        "updatedAt":       { "type": "string", "format": "date-time" }
                    }
                },
                "CreateRecordingRequest": {
                    "type": "object",
                    "description": "Request body for registering a new recording. Field names are **snake_case**.",
                    "required": ["file_path", "file_name"],
                    "properties": {
                        "file_path":        { "type": "string", "description": "Absolute path to the recording file" },
                        "file_name":        { "type": "string", "example": "service-2025-01-19.mp4" },
                        "file_size":        { "type": "integer", "format": "int64", "default": 0, "description": "File size in bytes" },
                        "duration_seconds": { "type": "number",  "format": "double",  "default": 0.0 },
                        "custom_title":     { "type": "string",  "description": "Optional custom YouTube title" }
                    }
                },
                "ConnectorStatus": {
                    "description": "Discriminated union representing the current connection state of a streaming connector. Discriminator field: `type`.",
                    "oneOf": [
                        {
                            "type": "object",
                            "title": "Disconnected",
                            "required": ["type"],
                            "properties": {
                                "type": { "type": "string", "enum": ["disconnected"] }
                            }
                        },
                        {
                            "type": "object",
                            "title": "Connecting",
                            "description": "A connection attempt is in progress.",
                            "required": ["type"],
                            "properties": {
                                "type": { "type": "string", "enum": ["connecting"] }
                            }
                        },
                        {
                            "type": "object",
                            "title": "Connected",
                            "required": ["type"],
                            "properties": {
                                "type": { "type": "string", "enum": ["connected"] }
                            }
                        },
                        {
                            "type": "object",
                            "title": "Error",
                            "description": "The last connection attempt failed.",
                            "required": ["type", "message"],
                            "properties": {
                                "type":    { "type": "string", "enum": ["error"] },
                                "message": { "type": "string", "description": "Human-readable error description" }
                            }
                        }
                    ],
                    "discriminator": {
                        "propertyName": "type"
                    }
                },
                "ConnectorStatuses": {
                    "type": "object",
                    "description": "Current status of all connectors.",
                    "required": ["obs", "vmix", "atem", "broadlink", "youtube", "facebook", "discord", "szentiras"],
                    "properties": {
                        "obs":       { "$ref": "#/components/schemas/ConnectorStatus" },
                        "vmix":      { "$ref": "#/components/schemas/ConnectorStatus" },
                        "atem":      { "$ref": "#/components/schemas/ConnectorStatus" },
                        "broadlink": { "$ref": "#/components/schemas/ConnectorStatus" },
                        "youtube":   { "$ref": "#/components/schemas/ConnectorStatus" },
                        "facebook":  { "$ref": "#/components/schemas/ConnectorStatus" },
                        "discord":   { "$ref": "#/components/schemas/ConnectorStatus" },
                        "szentiras": { "$ref": "#/components/schemas/ConnectorStatus" }
                    }
                },
                "BiblePassage": {
                    "type": "object",
                    "description": "A Bible passage normalised across both upstream APIs.",
                    "required": ["label", "verses"],
                    "properties": {
                        "label": { "type": "string", "example": "János 3,16" },
                        "verses": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["chapter", "verse", "text"],
                                "properties": {
                                    "chapter": { "type": "integer" },
                                    "verse":   { "type": "integer" },
                                    "text":    { "type": "string", "description": "Verse text with all markup stripped" }
                                }
                            }
                        }
                    }
                },
                "BibleSuggestion": {
                    "type": "object",
                    "required": ["cat", "label", "link"],
                    "properties": {
                        "cat":   { "type": "string", "example": "ref" },
                        "label": { "type": "string", "example": "1Móz 1" },
                        "link":  { "type": "string", "description": "Reference to pass back as the `reference` parameter" }
                    }
                },
                "ConnectorConfig": {
                    "description": "Configuration for one connector. The shape depends on the connector: `obs` (enabled, host, port, password), `vmix`/`atem` (enabled, host, port), `broadlink` (enabled), `youtube` (enabled, clientId, clientSecret), `facebook` (enabled, appId, appSecret, pageId), `discord` (enabled, webhookUrl), `szentiras` (enabled, apiKey).",
                    "type": "object",
                    "required": ["enabled"],
                    "properties": {
                        "enabled":      { "type": "boolean" },
                        "host":         { "type": "string" },
                        "port":         { "type": "integer" },
                        "password":     { "type": "string", "nullable": true, "description": "Write-only: reads return an empty string" },
                        "clientId":     { "type": "string" },
                        "clientSecret": { "type": "string", "description": "Write-only: reads return an empty string" },
                        "appId":        { "type": "string" },
                        "appSecret":    { "type": "string", "description": "Write-only: reads return an empty string" },
                        "pageId":       { "type": "string" },
                        "webhookUrl":   { "type": "string", "description": "Write-only: reads return an empty string" },
                        "apiKey":       { "type": "string", "description": "szentiras.eu API key, sent upstream as X-API-Key. Write-only: reads return an empty string." },
                        "passwordSet":    { "type": "boolean", "readOnly": true, "description": "Whether a password is stored" },
                        "clientSecretSet":{ "type": "boolean", "readOnly": true, "description": "Whether a client secret is stored" },
                        "appSecretSet":   { "type": "boolean", "readOnly": true, "description": "Whether an app secret is stored" },
                        "webhookUrlSet":  { "type": "boolean", "readOnly": true, "description": "Whether a webhook URL is stored" },
                        "apiKeySet":      { "type": "boolean", "readOnly": true, "description": "Whether an API key is stored. Send false to clear it." }
                    }
                },
                "ObsStreamSettings": {
                    "type": "object",
                    "description": "The RTMP destination OBS streams to.",
                    "required": ["server", "key"],
                    "properties": {
                        "serviceType": { "type": "string", "description": "OBS service type, e.g. `rtmp_custom` (response only)" },
                        "server":      { "type": "string", "example": "rtmp://a.rtmp.youtube.com/live2" },
                        "key":         { "type": "string" }
                    }
                },
                "WsConnectedMessage": {
                    "type": "object",
                    "description": "Sent once immediately after a WebSocket connection is established.",
                    "required": ["type", "serverId"],
                    "properties": {
                        "type":     { "type": "string", "enum": ["connected"] },
                        "serverId": { "type": "string", "format": "uuid", "description": "Unique server ID — regenerated on every server restart" }
                    }
                },
                "WsConnectorStatusMessage": {
                    "type": "object",
                    "description": "Pushed on connect (initial snapshot) and whenever a connector's state changes.",
                    "required": ["type", "connector", "status"],
                    "properties": {
                        "type":      { "type": "string", "enum": ["connector.status"] },
                        "connector": { "type": "string", "enum": ["obs", "vmix"] },
                        "status":    { "$ref": "#/components/schemas/ConnectorStatus" }
                    }
                },
                "WsEventChangedMessage": {
                    "type": "object",
                    "description": "Broadcast when an event row is inserted, updated, or deleted.",
                    "required": ["type", "data"],
                    "properties": {
                        "type": { "type": "string", "enum": ["event.changed"] },
                        "data": {
                            "type": "object",
                            "required": ["operation", "record"],
                            "properties": {
                                "operation": { "type": "string", "enum": ["INSERT", "UPDATE", "DELETE"] },
                                "record":    { "$ref": "#/components/schemas/Event" }
                            }
                        }
                    }
                },
                "WsRecordingChangedMessage": {
                    "type": "object",
                    "description": "Broadcast when a recording row is inserted or updated.",
                    "required": ["type", "data"],
                    "properties": {
                        "type": { "type": "string", "enum": ["recording.changed"] },
                        "data": {
                            "type": "object",
                            "required": ["operation", "record"],
                            "properties": {
                                "operation": { "type": "string", "enum": ["INSERT", "UPDATE", "DELETE"] },
                                "record":    { "$ref": "#/components/schemas/Recording" }
                            }
                        }
                    }
                }
            }
        },
        "paths": {
            "/api/events": {
                "get": {
                    "tags": ["Events"],
                    "summary": "List events",
                    "description": "Returns all events ordered by date (newest first). Each item includes a recording count but omits large text fields — use *Get event* to fetch the full record.",
                    "operationId": "listEvents",
                    "responses": {
                        "200": {
                            "description": "Array of event summaries",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/EventSummary" }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized — missing or invalid token" },
                        "500": { "description": "Database error" }
                    }
                },
                "post": {
                    "tags": ["Events"],
                    "summary": "Create event",
                    "operationId": "createEvent",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/CreateEventRequest" },
                                "example": {
                                    "title": "Sunday Service",
                                    "date_time": "2025-01-19T10:00:00Z",
                                    "speaker": "Pastor Smith",
                                    "bible_references": [
                                        { "type": "textus", "reference": "John 3:16", "translation": "UF" }
                                    ]
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Event created — returns the full event record",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Event" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "500": { "description": "Database error" }
                    }
                }
            },
            "/api/events/{id}": {
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "description": "Event UUID",
                        "schema": { "type": "string", "format": "uuid" }
                    }
                ],
                "get": {
                    "tags": ["Events"],
                    "summary": "Get event",
                    "description": "Returns the complete event record including all text fields.",
                    "operationId": "getEvent",
                    "responses": {
                        "200": {
                            "description": "Full event record",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Event" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "404": { "description": "Event not found" },
                        "500": { "description": "Database error" }
                    }
                },
                "put": {
                    "tags": ["Events"],
                    "summary": "Update event",
                    "description": "Replaces all fields of an existing event. This is a full replacement — omitted optional fields revert to their defaults (empty string, `\"UF\"`, `\"private\"`, `false`).",
                    "operationId": "updateEvent",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/CreateEventRequest" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Updated event record",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Event" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "404": { "description": "Event not found" },
                        "500": { "description": "Database error" }
                    }
                }
            },
            "/api/events/{id}/recordings": {
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "description": "Event UUID",
                        "schema": { "type": "string", "format": "uuid" }
                    }
                ],
                "get": {
                    "tags": ["Recordings"],
                    "summary": "List recordings",
                    "description": "Returns all recording files attached to an event, ordered by detection time (newest first).",
                    "operationId": "listRecordings",
                    "responses": {
                        "200": {
                            "description": "Array of recordings",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/Recording" }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "500": { "description": "Database error" }
                    }
                },
                "post": {
                    "tags": ["Recordings"],
                    "summary": "Add recording",
                    "description": "Registers a new recording file for an event. The file must already exist on the server's filesystem.",
                    "operationId": "createRecording",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/CreateRecordingRequest" },
                                "example": {
                                    "file_path": "/recordings/service-2025-01-19.mp4",
                                    "file_name": "service-2025-01-19.mp4",
                                    "file_size": 1073741824,
                                    "duration_seconds": 3600.0
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Recording registered — returns the full recording record",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Recording" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "500": { "description": "Database error" }
                    }
                }
            },
            "/api/presenter/parse": {
                "post": {
                    "tags": ["Presenter"],
                    "summary": "Parse a .pptx file",
                    "description": "Opens a `.pptx` file from the local filesystem, extracts text content from every slide, and returns the structured data. Only Open XML (`.pptx`) format is supported; legacy binary `.ppt` files must be re-saved as `.pptx` first.",
                    "operationId": "parsePresentation",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["filePath"],
                                    "properties": {
                                        "filePath": { "type": "string", "example": "/Users/admin/Presentations/sunday-service.pptx" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Parsed presentation data",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": { "type": "boolean", "example": true },
                                            "data": { "$ref": "#/components/schemas/ParsedPresentation" }
                                        }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "422": { "description": "File not found, not a valid .pptx, or no slides found" }
                    }
                }
            },
            "/api/bible/verses": {
                "get": {
                    "tags": ["Bible"],
                    "summary": "Look up a Bible passage",
                    "description": "Fetches a passage from the upstream Bible API that matches the translation (`*_v2` codes use the V2 API, everything else szentiras.eu) and returns it in one normalised shape. The core performs the upstream request, so UIs need no CORS workaround. szentiras.eu lookups send the API key from the `szentiras` connector config; without a valid key they fail with 502.",
                    "operationId": "getBiblePassage",
                    "parameters": [
                        {
                            "name": "reference",
                            "in": "query",
                            "required": true,
                            "description": "Passage reference, e.g. `Jn 3,16`",
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "translation",
                            "in": "query",
                            "required": true,
                            "description": "Translation code",
                            "schema": { "type": "string", "enum": ["UF_v2", "RUF_v2", "RUF", "KG", "KNB", "SZIT", "BD", "STL"] }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "The passage",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/BiblePassage" }
                                }
                            }
                        },
                        "400": { "description": "Missing reference or translation" },
                        "401": { "description": "Unauthorized" },
                        "502": { "description": "The upstream Bible API failed or returned something unparseable" }
                    }
                }
            },
            "/api/bible/suggest": {
                "get": {
                    "tags": ["Bible"],
                    "summary": "Autocomplete Bible references",
                    "description": "Reference suggestions from szentiras.eu. This endpoint is public upstream, so it works without an API key. Terms shorter than 2 characters return an empty list without calling upstream.",
                    "operationId": "getBibleSuggestions",
                    "parameters": [
                        {
                            "name": "term",
                            "in": "query",
                            "required": true,
                            "description": "Partial reference the user has typed",
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Matching references",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/BibleSuggestion" }
                                    }
                                }
                            }
                        },
                        "400": { "description": "Missing term" },
                        "401": { "description": "Unauthorized" },
                        "502": { "description": "The upstream Bible API failed or returned something unparseable" }
                    }
                }
            },
            "/api/connectors/status": {
                "get": {
                    "tags": ["Connectors"],
                    "summary": "Get connector statuses",
                    "description": "Returns the current connection state of every connector. Each value is a discriminated union on `type` — see the `ConnectorStatus` schema.",
                    "operationId": "getConnectorStatuses",
                    "responses": {
                        "200": {
                            "description": "Current connector statuses",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ConnectorStatuses" },
                                    "example": {
                                        "obs":  { "type": "connected" },
                                        "vmix": { "type": "disconnected" }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" }
                    }
                }
            },
            "/api/connectors/{name}/config": {
                "parameters": [
                    {
                        "name": "name",
                        "in": "path",
                        "required": true,
                        "description": "Connector name",
                        "schema": { "type": "string", "enum": ["obs", "vmix", "atem", "broadlink", "youtube", "facebook", "discord", "szentiras"] }
                    }
                ],
                "get": {
                    "tags": ["Connectors"],
                    "summary": "Get connector configuration",
                    "description": "Returns the stored configuration for one connector, or its defaults when nothing has been saved yet.\n\n**Secrets are never returned here.** `password`, `clientSecret`, `appSecret`, `apiKey` and `webhookUrl` always come back empty, with a companion boolean (`apiKeySet`, `passwordSet`, …) telling you whether one is stored. The host running the server can read them back through `/api/connectors/{name}/config/secrets`.",
                    "operationId": "getConnectorConfig",
                    "responses": {
                        "200": {
                            "description": "Stored configuration",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ConnectorConfig" },
                                    "example": { "enabled": true, "host": "localhost", "port": 4455, "password": null }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "404": { "description": "Unknown connector" }
                    }
                },
                "put": {
                    "tags": ["Connectors"],
                    "summary": "Save connector configuration",
                    "description": "Persists the configuration and applies it: OBS reconnects (or disconnects when `enabled` is false), YouTube and Facebook refresh the config used by the OAuth routes and stop when disabled.\n\n**Secret handling:** send a non-empty secret to replace it, leave it empty or omit it to keep the stored one, or send `\"<field>Set\": false` to clear it. This lets a client save a config whose secrets it was never allowed to read.",
                    "operationId": "putConnectorConfig",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/ConnectorConfig" }
                            }
                        }
                    },
                    "responses": {
                        "204": { "description": "Saved" },
                        "400": { "description": "Body does not match the connector's config shape" },
                        "401": { "description": "Unauthorized" },
                        "404": { "description": "Unknown connector" }
                    }
                }
            },
            "/api/connectors/{name}/config/secrets": {
                "get": {
                    "tags": ["Connectors"],
                    "summary": "Read a connector's stored secrets (host only)",
                    "description": "Returns the connector config **including** its secrets.\n\nRestricted to the desktop app hosting this server: it requires the normal auth token, an `X-Admin-Token` header matching the running server's admin token (regenerated every run, delivered to the host window over Tauri IPC, never over the network), and a request originating from loopback. Remote clients cannot obtain the admin token, and a leaked one is unusable off-host.\n\nConnectors that store no credentials (`vmix`, `atem`, `broadlink`) return 204.",
                    "operationId": "revealConnectorSecrets",
                    "parameters": [
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string", "enum": ["obs", "youtube", "facebook", "discord", "szentiras"] }
                        },
                        {
                            "name": "X-Admin-Token",
                            "in": "header",
                            "required": true,
                            "description": "The running server's admin token",
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "The config with secrets in the clear",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ConnectorConfig" }
                                }
                            }
                        },
                        "204": { "description": "This connector stores no secrets" },
                        "401": { "description": "Unauthorized" },
                        "403": { "description": "Missing/invalid admin token, or the request did not come from loopback" },
                        "404": { "description": "Unknown connector" }
                    }
                }
            },
            "/api/connectors/obs/connect": {
                "post": {
                    "tags": ["Connectors"],
                    "summary": "Connect OBS",
                    "description": "Starts the OBS connector using the stored configuration.",
                    "operationId": "connectObs",
                    "responses": {
                        "204": { "description": "Connection attempt started" },
                        "401": { "description": "Unauthorized" }
                    }
                }
            },
            "/api/connectors/obs/disconnect": {
                "post": {
                    "tags": ["Connectors"],
                    "summary": "Disconnect OBS",
                    "operationId": "disconnectObs",
                    "responses": {
                        "204": { "description": "Disconnected" },
                        "401": { "description": "Unauthorized" }
                    }
                }
            },
            "/api/connectors/obs/stream-settings": {
                "get": {
                    "tags": ["Connectors"],
                    "summary": "Get OBS stream destination",
                    "operationId": "getObsStreamSettings",
                    "responses": {
                        "200": {
                            "description": "Current RTMP destination",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ObsStreamSettings" }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized" },
                        "409": { "description": "OBS is not connected" },
                        "502": { "description": "OBS rejected the request" }
                    }
                },
                "put": {
                    "tags": ["Connectors"],
                    "summary": "Set OBS stream destination",
                    "description": "Applies a custom RTMP destination (`rtmp_custom`) to OBS.",
                    "operationId": "setObsStreamSettings",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/ObsStreamSettings" }
                            }
                        }
                    },
                    "responses": {
                        "204": { "description": "Applied" },
                        "401": { "description": "Unauthorized" },
                        "409": { "description": "OBS is not connected" },
                        "502": { "description": "OBS rejected the request" }
                    }
                }
            },
            "/ws": {
                "get": {
                    "tags": ["WebSocket"],
                    "summary": "WebSocket live stream",
                    "description": "**This endpoint performs a WebSocket upgrade — it cannot be tested with the HTTP \"Send\" button.**\n\nUse a dedicated WebSocket client instead:\n- [Hoppscotch](https://hoppscotch.io) → New request → WebSocket\n- [websocat](https://github.com/vi/websocat): `websocat 'ws://<host>/ws?token=<token>'`\n- Bruno: add a request with type `socket`\n\n---\n\n**Connection URL:** `ws://<host>/ws?token=<token>`\n\nAuthentication uses the same bearer token passed as a **query parameter** (the `Authorization` header is not available during the WebSocket handshake).\n\n### Initial messages (pushed immediately on connect)\n\n```json\n{ \"type\": \"connected\", \"serverId\": \"<uuid>\" }\n{ \"type\": \"connector.status\", \"connector\": \"obs\",  \"status\": { \"type\": \"connected\" } }\n{ \"type\": \"connector.status\", \"connector\": \"vmix\", \"status\": { \"type\": \"disconnected\" } }\n```\n\n### Broadcast messages (sent when data changes)\n\n| `type` | Trigger | Schema |\n|---|---|---|\n| `connector.status` | OBS or VMix connection state changes | `WsConnectorStatusMessage` |\n| `event.changed` | Event created, updated, or deleted | `WsEventChangedMessage` |\n| `recording.changed` | Recording created or updated | `WsRecordingChangedMessage` |\n| `presenter.state` | Presentation loaded or unloaded | `{ type, state: PresenterState }` with `renderMode: \"text\" | \"svg\"` |\n| `presenter.slide_changed` | Slide navigation | `{ type, currentSlide, totalSlides }` |\n\n### Presenter WS commands\n\n| Command | Fields | Description |\n|---|---|---|\n| `presenter.load` | `file_path`, optional `render_mode` | Load a .pptx into the presenter; defaults to text mode |\n| `presenter.load_bible_reference` | optional `event_id`, `reference_type` | Load an event Textus/Lekció Bible reference into the text presenter. If `event_id` is omitted, the backend-selected presenter event is used. |\n| `presenter.unload` | — | Clear the active presentation |\n| `presenter.next` | — | Advance one slide |\n| `presenter.prev` | — | Go back one slide |\n| `presenter.first` | — | Jump to slide 1 |\n| `presenter.last` | — | Jump to last slide |\n| `presenter.goto` | `slide` | Jump to a specific slide number |\n| `presenter.status` | — | Reply to requesting client with `presenter.state` |\n\n### Event WS commands for presenter controls\n\n| Command | Fields | Description |\n|---|---|---|\n| `events.presenter_list` | — | Return backend-ordered presenter event choices plus `selectedEventId` for the current/next event |\n\n`presentation.open` uses SVG mode by default in web-presenter mode. Send `render_mode: \"text\"` to force the text renderer.",
                    "operationId": "connectWebSocket",
                    "security": [],
                    "parameters": [
                        {
                            "name": "token",
                            "in": "query",
                            "required": true,
                            "description": "Bearer auth token",
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "101": { "description": "Switching Protocols — WebSocket handshake accepted (only reachable via a WebSocket client)" },
                        "426": {
                            "description": "Upgrade Required — returned when this endpoint is called as a plain HTTP request instead of a WebSocket upgrade",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "error":       { "type": "string", "example": "upgrade_required" },
                                            "description": { "type": "string" },
                                            "connect":     { "type": "string", "example": "ws://<host>/ws?token=<your-token>" },
                                            "auth":        { "type": "string" }
                                        }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized — token missing or invalid" }
                    }
                }
            }
        }
    })
}
