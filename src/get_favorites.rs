use clap::{ArgMatches, App};
use ronor::Sonos;
use super::Result;

pub const NAME: &'static str = "get-favorites";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get the list of Sonos favorites")
    .after_help("NOTE: Favorites do not include pinned items (any non-playable containers pinned to My Sonos) or Sonos playlists.")
}

pub fn run(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for favorite in sonos.get_favorites(&household)?.items.iter() {
        println!("{}", favorite.name);
      }
    }
    Ok(())
  })
}
