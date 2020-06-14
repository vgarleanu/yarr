use clap::{App, Arg};
use rustbg::search::Search;
use rustbg::Config;
use rustbg::extitem::ExtItem;
use std::fs::File;
use std::io::BufReader;
use std::sync::RwLock;
use std::io::{self, BufRead, Write};

const UAGENT: &str = "RarSpyder/1.0 (Linux x86_64;) Rust/1.44.0-nightly";

lazy_static::lazy_static! {
    static ref CONFIG: RwLock<Config> = {
        let home = std::env::var("HOME").unwrap();
        if let Ok(x) = File::open(format!("{}/.config/rarbg/config", home))
            .and_then(|x| Ok(BufReader::new(x))) {
                if let Ok(x) = serde_json::from_reader(x) {
                    return RwLock::new(x);
                }
            }

        let config = Config {
            cookie: String::new(),
            base_url: "rarbgproxied.org".to_string()
        };

        config.dump_config();

        RwLock::new(config)
    };
}

fn update_cookie(cookie: String) {
    if CONFIG.read().unwrap().cookie == cookie {
        return;
    }

    CONFIG.write().unwrap().cookie = cookie;
    CONFIG.write().unwrap().dump_config();
}

#[tokio::main]
async fn main() {
    let matches = App::new("Rustbg")
        .version("0.1")
        .author("Valerian G. <valerian.garleanu@pm.me>")
        .about("command-line interface for rarbg.")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .required(true)
                .help("Name or imdb tag you want to search for.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .help("Select media type")
                .possible_values(&["movies", "tv", "porn", "all"])
                .default_value("all")
                .required(false),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let mut search = Search::new("rarbgproxied.org".into(), CONFIG.read().unwrap().cookie.clone(), UAGENT.into());

    let to_search = matches.value_of("name").unwrap();
    println!("Searching for {}", to_search);

    let items = search.search(to_search.to_string()).await.unwrap();

    println!("Found {} results", items.len());

    for (idx, item) in items.iter().rev().enumerate() {
        println!(
            "[{}] {} ({} {}/{}) Uploaded by: {}",
            idx, item.name, item.size, item.seeds, item.leech, item.uploader
        );
    }

    print!("==> ");
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let input = stdin.lock().lines().next().unwrap().unwrap().parse::<usize>().unwrap();

    let item = items[input].clone();

    let mut ext = ExtItem::new(search.get_cookie(), "rarbgproxied.org".into(), item.id, UAGENT.into());
    ext.fetch().await.unwrap();

    println!("{}", ext.magnet);

    update_cookie(search.get_cookie());
}
