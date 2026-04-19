use crate::domain::SubscriberEmail;

pub struct EmailClient {
    _http_client: reqwest::Client,
    _base_url: String,
    _sender_email: SubscriberEmail,
}

impl EmailClient {
    pub fn new(_base_url: String, _sender_email: SubscriberEmail) -> Self {
        Self {
            _http_client: reqwest::Client::new(),
            _base_url,
            _sender_email,
        }
    }

    pub fn send_email(
        &self,
        _recipient_email: String,
        _subject_line: String,
        _html_content: String,
        _txt_content: String,
    ) -> Result<(), String> {
        todo!()
    }
}
/**
 * TODO:
 * 1. Add `tests_send_email_fires_request_to_base_url()`
 * 2. Implement send_email sketch
 *      - http_client.post(url)
 *          - Initial with string then with `reqwest.Uri`
 *      - `SendEmailRequestBody`
 *      - `authorization_token`, update unnittest, main and health_check integration test
 *          - update `EmailClientSettings config.rs as well
 *          - update configuration/base.yaml and local.yaml
 *      - Deal appropriately with SecretString and Faker
 * */
#[cfg(test)]
mod tests {}
