use crate::domain::{subscriber_email::SubscriberEmail, subscriber_username::SubscriberUsername};

pub struct NewSubscriber {
    pub username: SubscriberUsername,
    pub email: SubscriberEmail,
}
