use clap::{ArgMatches, App};
use oauth2::{ClientId, ClientSecret, RedirectUrl};
use ronor::Sonos;
use rustyline::Editor;
use super::Result;
use url::Url;

pub fn build() -> App<'static, 'static> {
  App::new("init")
    .about("Initialise sonos integration configuration")
}

pub fn run(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  println!("Go to https://integration.sonos.com/ and create an account.");
  println!("");
  println!("Create a new control integration.");
  println!("");
  let mut console = Editor::<()>::new();
  let client_id = ClientId::new(console.readline("Client identifier: ")?);
  let client_secret = ClientSecret::new(console.readline("Client secret: ")?);
  let redirect_url = RedirectUrl::new(
    Url::parse(&console.readline("Redirection URL: ")?)?
  );
  sonos.set_integration_config(client_id, client_secret, redirect_url)?;
  println!("");
  println!("OK, we're ready to go.");
  println!("Now run 'ronor login' to authorize access to your Sonos user account.");
  Ok(())
}

