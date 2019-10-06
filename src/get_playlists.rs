use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("get-playlists")
    .about("Get list of playlists")
}

pub fn run(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for playlist in sonos.get_playlists(&household)?.playlists.iter() {
        println!("{}", playlist.name);
      }
    }
    Ok(())
  })
}

