use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{Result, ArgMatchesExt};

pub const NAME: &str = "get-playlist";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get list of tracks contained in a playlist")
    .arg(super::household_arg())
    .arg(Arg::with_name("PLAYLIST").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let playlist = matches.playlist(sonos, &household)?;
  for track in sonos.get_playlist(&household, &playlist)?.tracks.iter() {
    match &track.album {
      Some(album) => println!("{} - {} - {}",
                              &track.name, &track.artist, album),
      None => println!("{} - {}",
                       &track.name, &track.artist)
    }
  }
  Ok(())
}
