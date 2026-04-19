use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender_email: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender_email: SubscriberEmail,
        authorization_token: SecretString,
    ) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            sender_email,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient_email: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let req_url = format!("{}/email", self.base_url);
        let req_body = SendEmailRequestBody {
            from: self.sender_email.as_ref().to_owned(),
            to: recipient_email.as_ref().to_owned(),
            subject: subject.to_owned(),
            text_body: text_content.to_owned(),
            html_body: html_content.to_owned(),
        };
        let _req_builder = self
            .http_client
            .post(&req_url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&req_body)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequestBody {
    from: String,
    to: String,
    subject: String,
    text_body: String,
    html_body: String,
}

#[cfg(test)]
mod tests {

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use secrecy::SecretString;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::any};

    #[tokio::test]
    async fn send_email_fires_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender_email,
            // Updated secrecy does not have just `Secret`
            SecretString::from(Faker.fake::<String>()),
        );

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
