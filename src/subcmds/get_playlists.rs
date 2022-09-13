use crate::{ArgMatchesExt, Result};
use clap::{Command, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-playlists";

pub fn build() -> Command<'static> {
  Command::new(NAME)
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
