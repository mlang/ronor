use crate::{ArgMatchesExt, Result, ResultExt};
use clap::{App, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use url::Url;

pub const NAME: &str = "speak";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Send synthetic speech to a player")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("SCRAPE")
        .long("scrape")
        .takes_value(true)
        .value_name("URI")
        .help("Scrape a specific web resource instead of taking text from STDIN")
    )
    .arg(
      Arg::with_name("LANGUAGE")
        .short("l")
        .long("language")
        .takes_value(true)
        .help("What language is the text coming from STDIN")
    )
    .group(ArgGroup::with_name("SOURCE").args(&["SCRAPE", "LANGUAGE"]))
    .arg(
      Arg::with_name("WORDS_PER_MINUTE")
        .short("s")
        .long("speed")
        .takes_value(true)
        .default_value("250")
    )
    .arg(
      Arg::with_name("VOLUME")
        .short("v")
        .long("volume")
        .takes_value(true)
        .default_value("75")
    )
    .arg(
      Arg::with_name("PLAYER")
        .required(true)
        .help("Name of the player")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let player = matches.player(&targets.players)?;
  let mut args = vec![
    String::from("-w"),
    String::from("/dev/stdout"),
    String::from("--stdin"),
  ];
  let text = match matches.value_of("SCRAPE") {
    Some(uri) => match scrapers().get(uri) {
      Some(scraper) => {
        let (language, text) = scraper(uri)?;
        args.extend(vec![String::from("-v"), language]);
        Some(text)
      }
      None => return Err("Scrape URI not supported".into())
    },
    None => None
  };
  if let Some(language) = matches.value_of("LANGUAGE") {
    args.extend(vec![String::from("-v"), language.to_string()]);
  }
  if let Some(wpm) = matches.value_of("WORDS_PER_MINUTE") {
    args.extend(vec![String::from("-s"), wpm.to_string()]);
  }
  if let Some(volume) = matches.value_of("VOLUME") {
    let volume = volume.parse::<u8>()? * 2;
    args.extend(vec![String::from("-a"), volume.to_string()]);
  }

  let espeak = if text.is_some() {
    Command::new("espeak")
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
  } else {
    Command::new("espeak")
      .args(args)
      .stdout(Stdio::piped())
      .spawn()
  }
  .chain_err(|| "Failed to spawn 'espeak'")?;
  if let Some(text) = text {
    espeak.stdin.unwrap().write_all(text.as_bytes())?;
    print!("{}", &text);
  }
  let mp3 = Command::new("ffmpeg")
    .args(&["-i", "-", "-v", "fatal", "-b:a", "96k", "-f", "mp3", "-"])
    .stdin(espeak.stdout.unwrap())
    .output()
    .chain_err(|| "Failed to spawn 'ffmpeg'")?
    .stdout;
  let client = reqwest::Client::new();
  let url = client
    .put("https://transfer.sh/espeak.mp3")
    .body(mp3)
    .send()
    .chain_err(|| "Failed to send audio clip to transfer.sh")?
    .error_for_status()
    .chain_err(|| "Failed to upload audio clip to transfer.sh")?
    .text()?;
  let url = Url::parse(&url).chain_err(|| "Failed to parse transfer.sh reply")?;
  sonos.load_audio_clip(
    &player,
    "guru.blind",
    "ping",
    None,
    None,
    None,
    None,
    Some(&url)
  )?;
  Ok(())
}

type Scraper = fn(&str) -> Result<(String, String)>;

fn parse(url: &str) -> Result<Html> {
  Ok(Html::parse_document(
    &reqwest::get(url)?.error_for_status()?.text()?
  ))
}

fn wetter_orf_at(uri: &str) -> Result<(String, String)> {
  let html = parse(&format!("https://{}/prognose", uri))?;
  let selector =
    Selector::parse("div.fulltextWrapper > h2, div.fulltextWrapper > p").unwrap();
  let mut s = String::new();
  let mut first_line = true;
  for element in html.select(&selector) {
    let is_h2 = element.value().name() == "h2";
    if is_h2 && !first_line {
      s += "\n";
    }
    s += &element.text().collect::<Vec<_>>().join("");
    s += "\n";
    if is_h2 {
      s += "\n";
    }
    first_line = false;
  }
  Ok(("de".to_string(), s))
}

fn zamg_ac_at(uri: &str) -> Result<(String, String)> {
  let html = parse(&format!("https://www.{}/", uri))?;
  let selector = Selector::parse("div#prognosenText > p").unwrap();
  let mut s = String::new();
  for element in html.select(&selector) {
    s += &element.text().collect::<Vec<_>>().join("");
    s += "\n";
  }
  Ok(("de".to_string(), s))
}

fn scrapers() -> HashMap<String, Scraper> {
  let mut m: HashMap<_, Scraper> = HashMap::new();
  for region in &[
    "burgenland",
    "kaernten",
    "niederoesterreich",
    "oberoesterreich",
    "salzburg",
    "steiermark",
    "tirol",
    "vorarlberg",
    "wien"
  ] {
    m.insert("wetter.orf.at/".to_string() + region, wetter_orf_at);
    m.insert(
      "zamg.ac.at/cms/de/wetter/wetter-oesterreich/".to_string() + region,
      zamg_ac_at
    );
  }
  m
}
