use octocrab::models::issues::{Comment, Issue};
use octocrab::models::webhook_events::payload::{IssueCommentWebhookEventAction, IssueCommentWebhookEventChanges};
use octocrab::Octocrab;
use regex::Regex;
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
            let regexes = vec![
                Regex::new(r"https?://(www\.)?(box|dropbox|mediafire|sugarsync|tresorit|hightail|opentext|sharefile|citrixsharefile|icloud|onedrive|1drv)\.com/[^\s)]+").unwrap(),
                Regex::new(r"https?://drive\.google\.com/[^\s)]+").unwrap(),
                Regex::new(r"https?://(www\.)?(bit\.ly|t\.co|tinyurl\.com|goo\.gl|ow\.ly|buff\.ly|is\.gd|soo\.gd|t2mio|bl\.ink|clck\.ru|shorte\.st|cutt\.ly|v\.gd|qr\.ae|rb\.gy|rebrand\.ly|tr\.im|shorturl\.at|lnkd\.in)/[^\s)]+").unwrap()
            ];

            for regex in regexes {
                if regex.is_match(body) {
                    if let Ok((owner, repo)) = extract_repository(payload.issue.repository_url.clone()) {
                        let installation = octocrab.apps()
                            .get_repository_installation(&owner, &repo)
                            .await
                            .unwrap();

                        let _ = octocrab
                            .installation(installation.id)
                            .issues(owner, repo)
                            .update_comment(payload.comment.id, &regex.replace(body, "[Potentially unsafe link removed]").to_string())
                            .await;
                    }
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