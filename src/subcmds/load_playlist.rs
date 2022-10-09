use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-playlist";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Load the specified playlist in a group")
    .arg(crate::household_arg())
    .arg(
      Arg::new("PLAY")
        .short('p')
        .long("play")
        .help("Automatically start playback")
    )
    .args(&crate::play_modes_args())
    .arg(
      Arg::new("PLAYLIST")
        .required(true)
        .help("The name of the playlist to load")
    )
    .arg(
      Arg::new("GROUP")
        .required(true)
        .help("The name of the group to load the playlist in")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let playlist = matches.playlist(sonos, &household)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let play_on_completion = matches.contains_id("PLAY");
  sonos.load_playlist(
    group,
    &playlist,
    play_on_completion,
    matches.play_modes().as_ref()
  )?;
  Ok(())
}
