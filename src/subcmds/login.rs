use crate::Result;
use clap::{Command, Arg, ArgMatches};
use oauth2::AuthorizationCode;
use ronor::Sonos;
use rustyline::Editor;
use std::process;

pub const NAME: &str = "login";

pub fn build() -> Command<'static> {
  Command::new(NAME)
    .about("Login with your sonos user account and authorize ronor")
    .arg(
      Arg::new("BROWSER")
        .default_value("lynx")
        .help("The browser to use to login to Sonos")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let (auth_url, csrf_token) = sonos.authorization_url()?;
  let _browser = process::Command::new(matches.get_one::<String>("BROWSER").unwrap())
    .arg(auth_url.as_str())
    .status()
    .expect("Failed to fire up browser.");
  println!("Token: {}", csrf_token.secret());
  let mut console = Editor::<()>::new();
  let code = console.readline("Code: ")?;
  sonos.authorize(AuthorizationCode::new(code.trim().to_string()))?;
  Ok(())
}
