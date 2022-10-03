use crate::{ArgMatchesExt, Result};
use clap::{Command, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-favorites";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Get the list of Sonos favorites")
    .after_help("NOTE: Favorites do not include pinned items (any non-playable containers pinned to My Sonos) or Sonos playlists.")
    .arg(crate::household_arg())
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  for favorite in sonos.get_favorites(&household)?.items.iter() {
    println!("{}", favorite.name);
  }
  Ok(())
}
