use crate::{ArgMatchesExt, Result};
use clap::{App, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-playlists";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get list of playlists")
    .arg(crate::household_arg())
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  for playlist in sonos.get_playlists(&household).await?.playlists.into_iter() {
    println!("{}", playlist.name);
  }
  Ok(())
}
