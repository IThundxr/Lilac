use hmac::{Hmac, Mac};
use rocket::data::FromData;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::tokio::io::AsyncReadExt;
use rocket::{data, Data, Request};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct GithubWebhookVerification<T> {
    pub payload: T
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned + Clone> FromData<'r> for GithubWebhookVerification<T> {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let signature = match req.headers().get_one("X-Hub-Signature-256") {
            Some(sig) => sig.to_string(),
            None => return data::Outcome::Error((Status::BadRequest, ())),
        };

        let mut payload: Vec<u8> = Vec::new();
        if let Err(_) = data.open(data::ByteUnit::max_value()).read_to_end(&mut payload).await {
            return data::Outcome::Error((Status::InternalServerError, ()));
        }

        let secret = match env::var("GITHUB_WEBHOOK_SECRET") {
            Ok(secret) => secret,
            Err(_) => return data::Outcome::Error((Status::InternalServerError, ()))
        };

        let mut hmac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(hmac) => hmac,
            Err(_) => return data::Outcome::Error((Status::InternalServerError, ()))
        };

        hmac.update(&payload);

        if format!("sha256={}", hex::encode(hmac.finalize().into_bytes())) == signature {
            match serde_json::from_slice::<T>(&payload) {
                Ok(parsed_payload) => {
                    data::Outcome::Success(GithubWebhookVerification {
                        payload: parsed_payload,
                    })
                }
                Err(_) => data::Outcome::Error((Status::UnprocessableEntity, ())),
            }
        } else {
            data::Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

pub enum GitHubEvent {
    IssueComment,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GitHubEvent {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<GitHubEvent, Self::Error> {
        let keys = request.headers().get("X-GitHub-Event").collect::<Vec<_>>();
        if keys.len() != 1 {
            return Outcome::Error((Status::BadRequest, ()));
        }

        let event = match keys[0] {
            "issue_comment" => GitHubEvent::IssueComment,
            _ => return Outcome::Error((Status::BadRequest, ()))
        };

        Outcome::Success(event)
    }
}
