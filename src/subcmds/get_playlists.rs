use clap::{ArgMatches, App};
use ronor::Sonos;
use crate::{Result, ArgMatchesExt};

pub const NAME: &str = "get-playlists";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get list of playlists")
    .arg(crate::household_arg())
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  for playlist in sonos.get_playlists(&household)?.playlists.iter() {
    println!("{}", playlist.name);
  }
  Ok(())
}
