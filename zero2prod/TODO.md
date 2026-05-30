# TODOS

### Error Handling

- [ ] Error Reporting For Operators

```JSON
{
  "timestamp": "2026-05-30T04:36:16.738017Z",
  "level": "ERROR",
  "fields": {
    "message": "Failed to execute store_token query: Database(PgDatabaseError { severity: Error, code: \"42703\", message: \"column \\\"subscription_token\\\" of relation \\\"subscription_tokens\\\" does not exist\", detail: None, hint: None, position: Some(Original(47)), where: None, schema: None, table: None, column: None, data_type: None, constraint: None, file: Some(\"parse_target.c\"), line: Some(1068), routine: Some(\"checkInsertTargets\") })"
  },
  "target": "zero2prod::routes::subscriptions",
  "span": {
    "subscriber_email": "lei_yin_loo@gmail.com",
    "subscriber_username": "lei yin",
    "name": "Adding a new subscriber"
  },
  "spans": [
    {
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:63169",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.target": "/subscriptions",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "request_id": "15ea41f4-dc67-459c-813a-e6ee8e9a5850",
      "name": "HTTP request"
    },
    {
      "subscriber_email": "lei_yin_loo@gmail.com",
      "subscriber_username": "lei yin",
      "name": "Adding a new subscriber"
    }
  ]
}

{
  "timestamp": "2026-05-30T04:36:16.738971Z",
  "level": "ERROR",
  "fields": {
    "message": "Error encountered while processing the incoming HTTP request: Failed to store confirmation token for a new subscriber.\n\nCause by:\n\tA database error was encountered when attempting to store a subscription token.Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not existCause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist"
  },
  "target": "tracing_actix_web::middleware",
  "span": {
    "exception.details": "Failed to store confirmation token for a new subscriber.\n\nCause by:\n\tA database error was encountered when attempting to store a subscription token.Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not existCause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist",
    "exception.message": "Failed to store confirmation token for a new subscriber.",
    "http.client_ip": "127.0.0.1",
    "http.flavor": "1.1",
    "http.host": "127.0.0.1:63169",
    "http.method": "POST",
    "http.route": "/{name}",
    "http.scheme": "http",
    "http.status_code": 500,
    "http.target": "/subscriptions",
    "http.user_agent": "",
    "otel.kind": "server",
    "otel.name": "POST /{name}",
    "otel.status_code": "ERROR",
    "request_id": "15ea41f4-dc67-459c-813a-e6ee8e9a5850",
    "name": "HTTP request"
  },
  "spans": [
    {
      "exception.details": "Failed to store confirmation token for a new subscriber.\n\nCause by:\n\tA database error was encountered when attempting to store a subscription token.Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not existCause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist",
      "exception.message": "Failed to store confirmation token for a new subscriber.",
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:63169",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.status_code": 500,
      "http.target": "/subscriptions",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "otel.status_code": "ERROR",
      "request_id": "15ea41f4-dc67-459c-813a-e6ee8e9a5850",
      "name": "HTTP request"
    }
  ]
}

```
