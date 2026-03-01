use axum::response::{Html, IntoResponse};
use axum::Json;
use serde_json::{json, Value};

const DOCS_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Sermon Helper API Reference</title>
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
            "title": "Sermon Helper API",
            "version": "1.0.0",
            "description": "REST API and WebSocket interface for the Sermon Helper desktop application.\n\n## Authentication\n\nAll `/api/*` endpoints require a **Bearer token** in the `Authorization` header:\n```\nAuthorization: Bearer <token>\n```\nThe token is displayed in the app's *Connection Guide* screen. It rotates on every server restart.\n\n## WebSocket — real-time push stream\n\n> **Note:** WebSocket is not an HTTP operation and cannot be tested from this page. Use a WebSocket client (e.g. [Hoppscotch](https://hoppscotch.io), [websocat](https://github.com/vi/websocat), or Bruno's socket type).\n\n**Endpoint:** `ws://<host>/ws?token=<token>`\n\nAuthentication uses the same bearer token passed as a **query parameter** (headers are not available during the WebSocket handshake).\n\n### Initial messages (sent immediately on connect)\n\n```json\n{ \"type\": \"connected\", \"serverId\": \"<uuid>\" }\n{ \"type\": \"connector.status\", \"connector\": \"obs\",  \"status\": { \"type\": \"connected\" } }\n{ \"type\": \"connector.status\", \"connector\": \"vmix\", \"status\": { \"type\": \"disconnected\" } }\n```\n\n### Push messages (broadcast on change)\n\n| `type` | Trigger | Schema |\n|---|---|---|\n| `connector.status` | OBS or VMix connection state changes | `WsConnectorStatusMessage` |\n| `event.changed` | Event created, updated, or deleted | `WsEventChangedMessage` |\n| `recording.changed` | Recording created or updated | `WsRecordingChangedMessage` |\n\n```json\n{ \"type\": \"connector.status\", \"connector\": \"obs\", \"status\": { \"type\": \"error\", \"message\": \"connection refused\" } }\n{ \"type\": \"event.changed\",     \"data\": { \"operation\": \"INSERT\", \"record\": { ...Event } } }\n{ \"type\": \"recording.changed\", \"data\": { \"operation\": \"UPDATE\", \"record\": { ...Recording } } }\n```\n\nFull payload definitions are in the `Ws*Message` schemas below."
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
            { "name": "Events",     "description": "Sermon / service events" },
            { "name": "Recordings", "description": "Video recording files linked to events" },
            { "name": "Connectors", "description": "Streaming software connector status (OBS, VMix)" },
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
                "Event": {
                    "type": "object",
                    "description": "Full event record including all text and metadata fields.",
                    "required": [
                        "id", "title", "dateTime", "speaker", "description",
                        "textus", "leckio", "textusTranslation", "leckioTranslation",
                        "youtubePrivacyStatus", "autoUploadEnabled", "createdAt", "updatedAt"
                    ],
                    "properties": {
                        "id":                   { "type": "string", "format": "uuid" },
                        "title":                { "type": "string", "example": "Sunday Morning Service" },
                        "dateTime":             { "type": "string", "format": "date-time", "description": "Scheduled date and time (ISO 8601 / UTC)" },
                        "speaker":              { "type": "string", "example": "Pastor Smith" },
                        "description":          { "type": "string" },
                        "textus":               { "type": "string", "description": "Main Bible text reference", "example": "John 3:16" },
                        "leckio":               { "type": "string", "description": "Lectio reading reference", "example": "Psalm 23" },
                        "textusTranslation":    { "type": "string", "description": "Bible translation abbreviation", "example": "UF" },
                        "leckioTranslation":    { "type": "string", "description": "Lectio translation abbreviation", "example": "UF" },
                        "youtubePrivacyStatus": {
                            "type": "string",
                            "enum": ["public", "unlisted", "private"],
                            "description": "YouTube upload privacy setting"
                        },
                        "autoUploadEnabled":    { "type": "boolean", "description": "Automatically upload recordings to YouTube when whitelisted" },
                        "createdAt":            { "type": "string", "format": "date-time" },
                        "updatedAt":            { "type": "string", "format": "date-time" }
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
                    "description": "Request body for creating or fully replacing an event. Field names are **snake_case**. Omitted optional fields default to empty string, `\"UF\"`, `\"private\"`, or `false` as appropriate.",
                    "required": ["title", "date_time"],
                    "properties": {
                        "title":                   { "type": "string", "example": "Sunday Morning Service" },
                        "date_time":               { "type": "string", "format": "date-time", "description": "Scheduled date and time" },
                        "speaker":                 { "type": "string", "default": "" },
                        "description":             { "type": "string", "default": "" },
                        "textus":                  { "type": "string", "default": "" },
                        "leckio":                  { "type": "string", "default": "" },
                        "textus_translation":      { "type": "string", "default": "UF", "description": "Bible translation abbreviation" },
                        "leckio_translation":      { "type": "string", "default": "UF" },
                        "youtube_privacy_status":  {
                            "type": "string",
                            "enum": ["public", "unlisted", "private"],
                            "default": "private"
                        },
                        "auto_upload_enabled":     { "type": "boolean", "default": false }
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
                    "description": "Current status of all streaming connectors.",
                    "required": ["obs", "vmix"],
                    "properties": {
                        "obs":  { "$ref": "#/components/schemas/ConnectorStatus" },
                        "vmix": { "$ref": "#/components/schemas/ConnectorStatus" }
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
                                    "textus": "John 3:16",
                                    "youtube_privacy_status": "private"
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
            "/api/connectors/status": {
                "get": {
                    "tags": ["Connectors"],
                    "summary": "Get connector statuses",
                    "description": "Returns the current connection state of OBS and VMix. Each value is a discriminated union on `type` — see the `ConnectorStatus` schema.",
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
            "/ws": {
                "get": {
                    "tags": ["WebSocket"],
                    "summary": "WebSocket live stream",
                    "description": "**This endpoint performs a WebSocket upgrade — it cannot be tested with the HTTP \"Send\" button.**\n\nUse a dedicated WebSocket client instead:\n- [Hoppscotch](https://hoppscotch.io) → New request → WebSocket\n- [websocat](https://github.com/vi/websocat): `websocat 'ws://<host>/ws?token=<token>'`\n- Bruno: add a request with type `socket`\n\n---\n\n**Connection URL:** `ws://<host>/ws?token=<token>`\n\nAuthentication uses the same bearer token passed as a **query parameter** (the `Authorization` header is not available during the WebSocket handshake).\n\n### Initial messages (pushed immediately on connect)\n\n```json\n{ \"type\": \"connected\", \"serverId\": \"<uuid>\" }\n{ \"type\": \"connector.status\", \"connector\": \"obs\",  \"status\": { \"type\": \"connected\" } }\n{ \"type\": \"connector.status\", \"connector\": \"vmix\", \"status\": { \"type\": \"disconnected\" } }\n```\n\n### Broadcast messages (sent when data changes)\n\n| `type` | Trigger | Schema |\n|---|---|---|\n| `connector.status` | OBS or VMix connection state changes | `WsConnectorStatusMessage` |\n| `event.changed` | Event created, updated, or deleted | `WsEventChangedMessage` |\n| `recording.changed` | Recording created or updated | `WsRecordingChangedMessage` |",
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
