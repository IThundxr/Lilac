use std::env;
use jsonwebtoken::EncodingKey;
use octocrab::models::AppId;
use octocrab::Octocrab;

pub struct App {
    pub octocrab: Octocrab
}

impl App {
    pub async fn new() -> Self {
        Self {
            octocrab: {
                let client_id = env::var("GITHUB_CLIENT_ID").unwrap().parse::<u64>().unwrap();
                let rsa_pem = env::var("GITHUB_PRIVATE_KEY").unwrap();

                let instance = Octocrab::builder()
                    .app(AppId(client_id), EncodingKey::from_rsa_pem(rsa_pem.as_bytes()).unwrap())
                    .build()
                    .unwrap();

                instance
            }
        }
    }
}