use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_playlist_by_name, Result};

pub const NAME: &str = "get-playlist";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get list of tracks contained in a playlist")
    .arg(Arg::with_name("PLAYLIST").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_playlist!(sonos, matches, playlist, {
      for household in sonos.get_households()?.iter() {
        for track in sonos.get_playlist(&household, &playlist)?.tracks.iter() {
          match &track.album {
            Some(album) => println!("{} - {} - {}",
                                    &track.name, &track.artist, album),
            None => println!("{} - {}",
                             &track.name, &track.artist)
          }
        }
      }
      Ok(())
    })
  })
}
