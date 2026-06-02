# TODOS

### Error Handling improvements

- [ ] Implement robust error log test on specific fields we expect to be part of the logs
- [ ] Implement reusable error handling module that can work for `confirm` handler as well


```JSON
{
  "timestamp": "2026-06-01T10:48:09.786042Z",
  "level": "ERROR",
  "fields": {
    "message": "Error encountered while processing the incoming HTTP request: Failed to store confirmation token for new subscriber.\n\n Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not exist Cause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist"
  },
  "target": "tracing_actix_web::middleware",
  "span": {
    "exception.details": "Failed to store confirmation token for new subscriber.\n\n Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not exist Cause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist",
    "exception.message": "Failed to store confirmation token for new subscriber.",
    "http.client_ip": "127.0.0.1",
    "http.flavor": "1.1",
    "http.host": "127.0.0.1:65499",
    "http.method": "POST",
    "http.route": "/{name}",
    "http.scheme": "http",
    "http.status_code": 500,
    "http.target": "/subscriptions",
    "http.user_agent": "",
    "otel.kind": "server",
    "otel.name": "POST /{name}",
    "otel.status_code": "ERROR",
    "request_id": "3fd546c5-3c88-41ac-8b48-87ba068167f9",
    "name": "HTTP request"
  },
  "spans": [
    {
      "exception.details": "Failed to store confirmation token for new subscriber.\n\n Cause by:\n\terror returned from database: column \"subscription_token\" of relation \"subscription_tokens\" does not exist Cause by:\n\tcolumn \"subscription_token\" of relation \"subscription_tokens\" does not exist",
      "exception.message": "Failed to store confirmation token for new subscriber.",
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:65499",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.status_code": 500,
      "http.target": "/subscriptions",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "otel.status_code": "ERROR",
      "request_id": "3fd546c5-3c88-41ac-8b48-87ba068167f9",
      "name": "HTTP request"
    }
  ]
}
```
