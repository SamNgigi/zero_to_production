use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    config::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};

pub async fn run_worker_until_stopped(config: Settings) -> Result<(), anyhow::Error> {
    let db_pool = get_connection_pool(&config.db);
    let email_client = config.email_client.client();
    worker_loop(db_pool, email_client).await
}

async fn worker_loop(db_pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_processing_task(&db_pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub enum ExecutionOutcome {
    EmptyQueue,
    TaskCompleted,
}

#[tracing::instrument(
    name = "Try Processing Task",
    skip_all,
    fields(
        newsletter_issue_id = tracing::field::Empty,
        subscriber_email = tracing::field::Empty,
    )
)]
pub async fn try_processing_task(
    db_pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let Some(task) = dequeue_task(db_pool).await? else {
        return Ok(ExecutionOutcome::EmptyQueue);
    };

    let (transaction, newsletter_issue_id, email) = task;
    match SubscriberEmail::parse(&email) {
        Ok(email) => {
            let issue = get_issue(db_pool, newsletter_issue_id).await?;
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.txt_content,
                    &issue.html_content,
                )
                .await
            {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver newsletter to confirmed subscriber. \
                     Skipping."
                );
            }
        }
        Err(e) => tracing::error!(
            error.cause_chain = ?e,
            error.message = %e,
            "Skipping confirmed subscriber. \
             Stored subscriber contact details were NOT valid."
        ),
    }

    delete_task(transaction, newsletter_issue_id, &email).await?;

    Ok(ExecutionOutcome::TaskCompleted)
}

type PgTransaction = Transaction<'static, Postgres>;

async fn delete_task(
    mut transaction: PgTransaction,
    newsletter_issue_id: Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
            DELETE FROM issue_delivery_queue
            WHERE
                newsletter_issue_id = $1 AND 
                subscriber_email = $2;
        "#,
        newsletter_issue_id,
        email
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    txt_content: String,
    html_content: String,
}

async fn get_issue(
    db_pool: &PgPool,
    newsletter_issue_id: Uuid,
) -> Result<NewsletterIssue, anyhow::Error> {
    let row = sqlx::query_as!(
        NewsletterIssue,
        r#"
            SELECT  title,
                    txt_content,
                    html_content
                FROM newsletter_issues
            WHERE newsletter_issue_id = $1;
        "#,
        newsletter_issue_id,
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row)
}

async fn dequeue_task(
    db_pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = db_pool.begin().await?;
    let row = sqlx::query!(
        r#"
            SELECT  newsletter_issue_id,
                    subscriber_email
                FROM issue_delivery_queue
            LIMIT 1
            FOR UPDATE
            SKIP LOCKED;
        "#,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(row) = row else { return Ok(None) };
    Ok(Some((
        transaction,
        row.newsletter_issue_id,
        row.subscriber_email,
    )))
}
