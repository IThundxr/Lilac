use octocrab::models::issues::{Comment, Issue};
use octocrab::models::webhook_events::payload::{IssueCommentWebhookEventAction, IssueCommentWebhookEventChanges};
use octocrab::Octocrab;
use regex::Regex;
use rocket::form::validate::Contains;
use rocket::serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueCommentWebhookEventPayload_ {
    pub action: IssueCommentWebhookEventAction,
    pub changes: Option<IssueCommentWebhookEventChanges>,
    pub comment: Comment,
    pub enterprise: Option<serde_json::Value>,
    pub issue: Issue,
}

pub async fn issue_comment(octocrab: &Octocrab, payload: IssueCommentWebhookEventPayload_) {
    if payload.action == IssueCommentWebhookEventAction::Created ||
        payload.action == IssueCommentWebhookEventAction::Edited {
        if let Some(body) = &payload.comment.body {
            if body.contains("mediafire") {
                if let Ok((owner, repo)) = extract_repository(payload.issue.repository_url.clone()) {
                    let installation = octocrab.apps()
                        .get_repository_installation(&owner, &repo)
                        .await
                        .unwrap();

                    let _ = octocrab
                        .installation(installation.id)
                        .issues(owner, repo)
                        .delete_comment(payload.comment.id)
                        .await;
                }
            }
        }
    }
}

pub fn extract_repository(url: Url) -> Result<(String, String), ()> {
    let regex = Regex::new("https://api.github.com/repos/(.*)/(.*)").unwrap();

    match regex.captures(url.as_str()) {
        Some(captures) => {
            let owner = captures.get(1).map_or(None, |m| Some(m.as_str().to_string()));
            let repo = captures.get(2).map_or(None, |m| Some(m.as_str().to_string()));

            if let Some(owner) = owner {
                if let Some(repo) = repo {
                    return Ok((owner, repo));
                }
            }
        }
        _ => ()
    }

    Err(())
}