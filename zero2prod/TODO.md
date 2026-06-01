# TODOS

### Error Handling

- [ ] Error Reporting For Operators


```JSON
{
  "timestamp": "2026-06-01T08:18:23.884688Z",
  "level": "ERROR",
  "fields": {
    "message": "Failed to execute query: Database(PgDatabaseError { severity: Error, code: \"42703\", message: \"column \\\"email\\\" of relation \\\"subscriptions\\\" does not exist\", detail: None, hint: None, position: Some(Original(45)), where: None, schema: None, table: None, column: None, data_type: None, constraint: None, file: Some(\"parse_target.c\"), line: Some(1068), routine: Some(\"checkInsertTargets\") })"
  },
  "target": "zero2prod::routes::subscriptions",
  "span": {
    "name": "Saving new subscriber details in the database"
  },
  "spans": [
    {
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:50832",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.target": "/subscriptions",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "request_id": "607d4ad5-1016-4bca-b724-73fa2b208f42",
      "name": "HTTP request"
    },
    {
      "subscriber_email": "lei_yin_loo@gmail.com",
      "subscriber_username": "lei yin",
      "name": "Adding a new subscriber"
    },
    {
      "name": "Saving new subscriber details in the database"
    }
  ]
}
{
  "timestamp": "2026-06-01T08:18:23.885615Z",
  "level": "ERROR",
  "fields": {
    "message": "Error encountered while processing the incoming HTTP request: Failed to insert new subscriber to database.\n\n Cause by:\n\terror returned from database: column \"email\" of relation \"subscriptions\" does not exist Cause by:\n\tcolumn \"email\" of relation \"subscriptions\" does not exist"
  },
  "target": "tracing_actix_web::middleware",
  "span": {
    "exception.details": "Failed to insert new subscriber to database.\n\n Cause by:\n\terror returned from database: column \"email\" of relation \"subscriptions\" does not exist Cause by:\n\tcolumn \"email\" of relation \"subscriptions\" does not exist",
    "exception.message": "Failed to insert new subscriber to database.",
    "http.client_ip": "127.0.0.1",
    "http.flavor": "1.1",
    "http.host": "127.0.0.1:50832",
    "http.method": "POST",
    "http.route": "/{name}",
    "http.scheme": "http",
    "http.status_code": 500,
    "http.target": "/subscriptions",
    "http.user_agent": "",
    "otel.kind": "server",
    "otel.name": "POST /{name}",
    "otel.status_code": "ERROR",
    "request_id": "607d4ad5-1016-4bca-b724-73fa2b208f42",
    "name": "HTTP request"
  },
  "spans": [
    {
      "exception.details": "Failed to insert new subscriber to database.\n\n Cause by:\n\terror returned from database: column \"email\" of relation \"subscriptions\" does not exist Cause by:\n\tcolumn \"email\" of relation \"subscriptions\" does not exist",
      "exception.message": "Failed to insert new subscriber to database.",
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:50832",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.status_code": 500,
      "http.target": "/subscriptions",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "otel.status_code": "ERROR",
      "request_id": "607d4ad5-1016-4bca-b724-73fa2b208f42",
      "name": "HTTP request"
    }
  ]
}
```
