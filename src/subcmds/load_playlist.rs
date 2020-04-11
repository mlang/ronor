use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-playlist";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Load the specified playlist in a group")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("PLAY")
        .short("p")
        .long("play")
        .help("Automatically start playback")
    )
    .args(&crate::play_modes_args())
    .arg(
      Arg::with_name("PLAYLIST")
        .required(true)
        .help("The name of the playlist to load")
    )
    .arg(
      Arg::with_name("GROUP")
        .required(true)
        .help("The name of the group to load the playlist in")
    )
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  let playlist = matches.playlist(sonos, &household).await?;
  let targets = sonos.get_groups(&household).await?;
  let group = matches.group(&targets.groups)?;
  let play_on_completion = matches.is_present("PLAY");
  sonos.load_playlist(
    &group,
    &playlist,
    play_on_completion,
    matches.play_modes().as_ref()
  ).await?;
  Ok(())
}
