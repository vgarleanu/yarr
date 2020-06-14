use fancy_regex::Regex;
use select::{document::Document, predicate::Name};
use std::io::Write;
use std::thread;
use std::time::Duration;

const UAGENT: &str = "RarSpyder/1.0 (Linux x86_64;) Rust/1.44.0-nightly";

lazy_static::lazy_static! {
    /// sk, c, i, r, r2
    static ref CAPS: Vec<Regex> = vec![
        Regex::new(r"(?<=var value_sk = ')(.*)(?=')").unwrap(),
        Regex::new(r"(?<=var value_c = ')(.*)(?=')").unwrap(),
        Regex::new(r"(?<=var value_i = ')(.*)(?=')").unwrap(),
        Regex::new(r"(?<=&r=)(\d+)(?=')").unwrap(),
        Regex::new(r#"(?<="&r=)(\d+)(?=")"#).unwrap(),
    ];
}

pub struct Captcha {
    base_url: String,

    /// rarbg captcha internal
    sk: String,
    /// rarbg captcha internal
    cid: String,
    /// rarbg captcha internal
    i: String,
    /// rarbg captcha internal
    r: String,
    /// rarbg captcha internal
    r2: String,
    /// rarbg captcha internal
    captcha_id: String,
}

impl Captcha {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            sk: String::new(),
            cid: String::new(),
            i: String::new(),
            r: String::new(),
            r2: String::new(),
            captcha_id: String::new(),
        }
    }

    /// Method `solve` solves the captcha returning the cookie required for operation
    pub async fn solve(&mut self) -> Result<Option<String>, reqwest::Error> {
        // Create a dummy request so that we get redirected to the captcha page
        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(UAGENT)
            .build()?
            .get(
                format!(
                    "https://{}/torrents.php?category[]=14,48,17,44,45,47",
                    self.base_url
                )
                .as_str(),
            )
            .query(&[("search", "blade runner 2049")])
            .send()
            .await?;

        if res.url().path() != "/threat_defence.php" {
            return Ok(None);
        }

        let document = Document::from(res.text().await.unwrap().as_ref());
        let script = document
            .find(Name("script"))
            .filter_map(|x| x.first_child().and_then(|y| y.as_text()))
            .collect::<Vec<_>>()
            .pop()
            .expect("No script tag found");

        let secrets = CAPS
            .iter()
            .map(|x| {
                x.captures(script)
                    .unwrap()
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
            })
            .collect::<Vec<_>>();

        self.sk = secrets[0].into();
        self.cid = secrets[1].into();
        self.i = secrets[2].into();
        self.r = secrets[3].into();
        self.r2 = secrets[4].into();

        self.generate_captcha().await?;
        self.get_captcha().await?;
        self.solve_captcha().await
    }

    // Function acts as the stage 2 of the captcha process
    async fn generate_captcha(&mut self) -> Result<(), reqwest::Error> {
        let _ = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(UAGENT)
            .build()?
            .get(format!("https://{}/threat_defence_ajax.php", self.base_url).as_str())
            .query(&[
                ("sk", self.sk.as_str()),
                ("cid", self.cid.as_str()),
                ("i", self.i.as_str()),
                ("r", self.r.as_str()),
            ])
            .header("Cookie", format!("sk={}", self.sk.as_str()))
            .send()
            .await?;

        // sleep for 3.5s as required by rarbg for the captcha img to generate
        thread::sleep(Duration::from_millis(3500));

        Ok(())
    }

    async fn get_captcha(&mut self) -> Result<(), reqwest::Error> {
        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(UAGENT)
            .build()?
            .get(format!("https://{}/threat_defence.php", self.base_url).as_str())
            .query(&[
                ("defence", "2"),
                ("sk", self.sk.as_str()),
                ("cid", self.cid.as_str()),
                ("i", self.i.as_str()),
                ("ref_cookie", self.base_url.as_str()),
                ("r", self.r2.as_str()),
            ])
            .header("Cookie", format!("sk={}", self.sk.as_str()))
            .send()
            .await?;

        let doc = Document::from(res.text().await?.as_ref());
        self.captcha_id = doc
            .find(Name("input"))
            .filter(|x| x.attr("name") == Some("captcha_id"))
            .filter_map(|x| x.attr("value"))
            .collect::<Vec<_>>()
            .pop()
            .unwrap()
            .to_string();

        let mut img_selector = doc.find(Name("img"));
        img_selector.next();
        let img = img_selector.next().unwrap().attr("src").unwrap();

        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(UAGENT)
            .build()
            .unwrap()
            .get(format!("https://rarbgproxied.org{}", img).as_str())
            .header("Cookie", format!("sk={}", self.sk.as_str()))
            .send()
            .await
            .unwrap();

        let mut file = std::fs::File::create(format!("{}.png", self.sk.as_str())).unwrap();
        let bytes = res.bytes().await.unwrap();
        file.write_all(&bytes).unwrap();

        Ok(())
    }

    async fn solve_captcha(&mut self) -> Result<Option<String>, reqwest::Error> {
        let mut lt = leptess::LepTess::new(None, "eng").unwrap();
        lt.set_image(format!("{}.png", self.sk.as_str()).as_str());
        lt.set_source_resolution(70);

        let code = lt.get_utf8_text().unwrap();

        let res = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(UAGENT)
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .get(format!("https://{}/threat_defence.php", self.base_url).as_str())
            .query(&[
                ("defence", "2"),
                ("sk", self.sk.as_str()),
                ("cid", self.cid.as_str()),
                ("i", self.i.as_str()),
                ("ref_cookie", "rarbgproxied.org"),
                ("r", self.r2.as_str()),
                ("solve_string", code.as_str().trim_end()),
                ("captcha_id", self.captcha_id.as_str()),
                ("submitted_bot_captcha", "1"),
            ])
            .header("Cookie", format!("sk={}", self.sk.as_str()))
            .send()
            .await?;

        Ok(Some(
            res.cookies()
                .map(|x| format!("{}={}", x.name(), x.value()))
                .collect::<Vec<_>>()
                .join("; "),
        ))
    }
}
