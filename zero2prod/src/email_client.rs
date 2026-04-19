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

    pub async fn send_email(
        &self,
        _recipient_email: SubscriberEmail,
        _subject: &str,
        _html_content: &str,
        _text_content: &str,
    ) -> Result<(), String> {
        Ok(()) // No matter the input
    }
}

#[cfg(test)]
mod tests {

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    use fake::{
        Fake,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::any};

    #[tokio::test]
    async fn send_email_fires_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender_email);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..3).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
