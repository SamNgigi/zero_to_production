#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    todo!()
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    create_confirmed_subscriber().await;
    todo!()
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    create_unconfirmed_subscriber().await;
    todo!()
}

async fn create_confirmed_subscriber() {
    todo!()
}

async fn create_unconfirmed_subscriber() {
    todo!()
}
