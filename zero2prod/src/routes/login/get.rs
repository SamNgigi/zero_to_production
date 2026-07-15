use actix_web::{HttpResponse, http::header::ContentType, web};
use hmac::{Hmac, KeyInit, Mac};
use secrecy::ExposeSecret;
use sha2::Sha256;

use crate::startup::HMACSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret_key: &HMACSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let error_query = format!("error={}", urlencoding::Encoded::new(&self.error));
        type Hmac256 = Hmac<Sha256>;
        let mut mac = Hmac256::new_from_slice(secret_key.0.expose_secret().as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(error_query.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

#[tracing::instrument(name = "Login Form", skip(secret_key, query))]
pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret_key: web::Data<HMACSecret>,
) -> HttpResponse {
    let error_html = match query {
        None => "".to_string(),
        Some(query_params) => match query_params.0.verify(&secret_key) {
            Ok(err_msg) => format!("<p>{}</p>", html_escape::encode_safe(&err_msg)),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify query parameters using the HMAC tag",
                );
                "".into()
            }
        },
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("./login.html"), error_html))
}
