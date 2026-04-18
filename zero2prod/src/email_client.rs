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
