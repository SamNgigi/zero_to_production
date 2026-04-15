use crate::domain::{SubscriberEmail, SubscriberUsername};

pub struct NewSubscriber {
    pub username: SubscriberUsername,
    pub email: SubscriberEmail,
}
