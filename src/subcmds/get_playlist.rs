use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-playlist";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Get list of tracks contained in a playlist")
    .arg(crate::household_arg())
    .arg(Arg::new("PLAYLIST").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let playlist = matches.playlist(sonos, &household)?;
  for track in sonos.get_playlist(&household, &playlist)?.tracks.iter() {
    match &track.album {
      Some(album) => println!("{} - {} - {}", &track.name, &track.artist, album),
      None => println!("{} - {}", &track.name, &track.artist)
    }
  }
  Ok(())
}
