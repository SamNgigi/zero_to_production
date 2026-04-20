use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender_email: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender_email: SubscriberEmail) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            sender_email,
        }
    }

    pub async fn send_email(
        &self,
        recipient_email: SubscriberEmail,
        subject_line: &str,
        html_content: &str,
        txt_content: &str,
    ) -> Result<(), String> {
        let req_body = SendEmailRequestBody {
            from: self.sender_email.as_ref().to_owned(),
            to: recipient_email.as_ref().to_owned(),
            subject: subject_line.to_owned(),
            html_content: html_content.to_owned(),
            text_content: txt_content.to_owned(),
        };
        let _req_builder = self.http_client.post(&self.base_url).json(&req_body);
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequestBody {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

/**
 * TODO:
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
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    use fake::{
        Fake,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::any};

    #[tokio::test]
    async fn tests_send_email_fires_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender_email);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..3).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(recipient_email, &subject, &content, &content)
            .await;
    }
}
