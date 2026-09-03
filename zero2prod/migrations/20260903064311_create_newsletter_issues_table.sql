-- Add migration script here
CREATE TABLE newsletter_issues (
  newsletter_issue_id UUID NOT NULL,
  PRIMARY KEY (newsletter_issue_id),
  title TEXT NOT NULL,
  txt_content TEXT NOT NULL,
  html_content TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL
);
