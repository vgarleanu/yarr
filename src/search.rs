use crate::captcha::Captcha;
use crate::item::Item;
use async_recursion::async_recursion;
use select::document::Document;
use select::predicate::Class;
use std::convert::TryFrom;

pub struct Search {
    base_url: String,
    cookie: String,
    uagent: String,
    captcha: Captcha,
}

impl Search {
    pub fn new(base_url: String, cookie: String, uagent: String) -> Self {
        Self {
            base_url: base_url.clone(),
            cookie,
            uagent,
            captcha: Captcha::new(base_url),
        }
    }

    pub fn get_cookie(&self) -> String {
        self.cookie.clone()
    }

    #[async_recursion(?Send)]
    pub async fn search(&mut self, name: String) -> Result<Vec<Item>, reqwest::Error> {
        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(self.uagent.as_str())
            .build()?
            .get(format!("https://{}/torrents.php?category=movies", self.base_url).as_str())
            .header("Cookie", self.cookie.as_str())
            .query(&[("search", name.as_str())])
            .send()
            .await?;

        if res.url().path() == "/threat_defence.php" {
            self.cookie = self.captcha.solve().await.unwrap().unwrap();
            return self.search(name).await;
        }

        let doc = Document::from(res.text().await.unwrap().as_ref());
        Ok(doc
            .find(Class("lista2"))
            .filter_map(|x| Item::try_from(x).ok())
            .collect::<Vec<_>>())
    }
}
