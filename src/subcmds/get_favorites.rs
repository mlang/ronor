use crate::{ArgMatchesExt, Result};
use clap::{App, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-favorites";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get the list of Sonos favorites")
    .after_help("NOTE: Favorites do not include pinned items (any non-playable containers pinned to My Sonos) or Sonos playlists.")
    .arg(crate::household_arg())
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  for favorite in sonos.get_favorites(&household).await?.items.into_iter() {
    println!("{}", favorite.name);
  }
  Ok(())
}
