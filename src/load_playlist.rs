use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{Result, ArgMatchesExt};

pub const NAME: &str = "load-playlist";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Load the specified playlist in a group")
    .arg(super::household_arg())
    .arg(Arg::with_name("PLAY").short("p").long("play")
           .help("Automatically start playback"))
    .arg(Arg::with_name("REPEAT").short("r").long("repeat"))
    .arg(Arg::with_name("REPEAT_ONE").short("o").long("repeat-one"))
    .arg(Arg::with_name("CROSSFADE").short("c").long("crossfade")
           .help("Do crossfade between tracks"))
    .arg(Arg::with_name("SHUFFLE").short("s").long("shuffle")
           .help("Shuffle the tracks"))
    .arg(Arg::with_name("PLAYLIST").required(true)
           .help("The name of the playlist to load"))
    .arg(Arg::with_name("GROUP").required(true)
           .help("The name of the group to load the playlist in"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let household = matches.household(sonos)?;
    let playlist = matches.playlist(sonos, &household)?;
    let targets = sonos.get_groups(&household)?;
    let group = matches.group(&targets.groups)?;
    let play_on_completion = matches.is_present("PLAY");
    sonos.load_playlist(&group,
      &playlist, play_on_completion, matches.play_modes().as_ref())?;
    Ok(())
  })
}
