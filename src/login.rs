use clap::{Arg, ArgMatches, App};
use oauth2::AuthorizationCode;
use ronor::Sonos;
use rustyline::Editor;
use std::process::{Command};
use super::Result;

pub const NAME: &str = "login";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Login with your sonos user account and authorize ronor")
    .arg(Arg::with_name("BROWSER").default_value("lynx")
           .help("The browser to use to login to Sonos"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let (auth_url, csrf_token) = sonos.authorization_url()?;
  let _browser = Command::new(matches.value_of("BROWSER").unwrap())
    .arg(auth_url.as_str())
    .status().expect("Failed to fire up browser.");
  println!("Token: {}", csrf_token.secret());
  let mut console = Editor::<()>::new();
  let code = console.readline("Code: ")?;
  sonos.authorize(AuthorizationCode::new(code.trim().to_string()))?;
  Ok(())
}
