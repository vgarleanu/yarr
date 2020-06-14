use crate::captcha::Captcha;
use async_recursion::async_recursion;
use select::document::Document;
use select::predicate::{And, Attr, Class, Name};

pub struct ExtItem {
    pub captcha: Captcha,
    pub cookie: String,
    pub base_url: String,
    pub uagent: String,
    pub id: String,
    pub magnet: String,
}

impl ExtItem {
    pub fn new(cookie: String, base_url: String, id: String, uagent: String) -> Self {
        Self {
            captcha: Captcha::new(base_url.clone()),
            cookie,
            base_url,
            id,
            uagent,
            magnet: String::new(),
        }
    }

    #[async_recursion(?Send)]
    pub async fn fetch(&mut self) -> Result<(), reqwest::Error> {
        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(self.uagent.as_str())
            .build()?
            .get(format!("https://{}{}", self.base_url, self.id).as_str())
            .header("Cookie", self.cookie.as_str())
            .send()
            .await?;

        if res.url().path() == "/threat_defence.php" {
            self.cookie = self.captcha.solve().await.unwrap().unwrap();
            return self.fetch().await;
        }

        let doc = Document::from(res.text().await.unwrap().as_ref());
        let links = doc
            .find(And(Class("lista-rounded"), Name("table")))
            .flat_map(|x| x.find(Attr("href", ())).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let mut link_iter = links.iter();
        link_iter.next();

        self.magnet = link_iter.next().unwrap().attr("href").unwrap().to_string();
        Ok(())
    }
}
