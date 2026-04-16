use axum::{
    Form,
    extract::{FromRequest, Request},
    http::StatusCode,
};

use crate::{
    domain::{subscriber_email::SubscriberEmail, subscriber_username::SubscriberUsername},
    routes::FormData,
};

pub struct NewSubscriber {
    pub username: SubscriberUsername,
    pub email: SubscriberEmail,
}

impl<S> FromRequest<S> for NewSubscriber
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the raw form first
        let Form(raw_form_data) = Form::<FormData>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // Apply conversion using TryFrom
        NewSubscriber::try_from(raw_form_data).map_err(|e| (StatusCode::BAD_REQUEST, e))
    }
}
