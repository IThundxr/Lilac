#[macro_use]
extern crate rocket;
mod webhook;
mod guards;
mod app;

use crate::guards::github::GitHubEvent;
use crate::webhook::issues::IssueCommentWebhookEventPayload_;
use dotenvy::dotenv;
use rocket::http::Status;
use rocket::State;
use crate::app::App;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    rocket::build()
        .manage(App::new().await)
        .mount("/", routes![github])
}

#[post("/webhook/github", data = "<data>")]
pub async fn github(app: &State<App>, event: Option<GitHubEvent>, data: guards::github::GithubWebhookVerification<IssueCommentWebhookEventPayload_>) -> Status {
    if let Some(event) = event {
        match event {
            GitHubEvent::IssueComment => {
                webhook::issues::issue_comment(&app.octocrab, data.payload).await;
            }
        }
    }

    Status::Ok
}