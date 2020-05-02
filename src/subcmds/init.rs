use crate::Result;
use clap::{App, ArgMatches};
use oauth2::{ClientId, ClientSecret, RedirectUrl};
use ronor::Sonos;
use rustyline::Editor;

pub const NAME: &str = "init";

pub fn build() -> App<'static, 'static> {
  App::new(NAME).about("Initialise sonos integration configuration")
}

pub fn run(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  println!("1. Go to https://integration.sonos.com/ and create a developer account.");
  println!("   NOTE that your existing Sonos user account does not work.");
  println!();
  println!("2. Create a new control integration and enter the information below.");
  println!();
  let mut console = Editor::<()>::new();
  let client_id = ClientId::new(console.readline("Client identifier: ")?);
  let client_secret = ClientSecret::new(console.readline("Client secret: ")?);
  let redirect_url =
    RedirectUrl::new(console.readline("Redirection URL: ")?)?;
  sonos.set_integration_config(client_id, client_secret, redirect_url)?;
  println!();
  println!("OK, ready to go.");
  println!("Now run 'ronor login' to authorize access to your Sonos user account.");
  Ok(())
}
