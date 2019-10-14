use clap::{Arg, ArgMatches, App};
use ronor::{Sonos, PlayModes};
use super::{find_favorite_by_name, find_group_by_name, Result, ErrorKind};

pub const NAME: &str = "load-favorite";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Load the specified favorite in a group")
    .arg(Arg::with_name("PLAY").short("p").long("play")
           .help("Automatically start playback"))
    .arg(Arg::with_name("REPEAT").short("r").long("repeat"))
    .arg(Arg::with_name("REPEAT_ONE").short("o").long("repeat-one"))
    .arg(Arg::with_name("CROSSFADE").short("c").long("crossfade")
           .help("Do crossfade between tracks"))
    .arg(Arg::with_name("SHUFFLE").short("s").long("shuffle")
           .help("Shuffle the tracks"))
    .arg(Arg::with_name("FAVORITE").required(true))
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_favorite!(sonos, matches, favorite, {
      with_group!(sonos, matches, group, {
        let play_on_completion = matches.is_present("PLAY");
        let play_modes = super::play_modes(matches);
        sonos.load_favorite(&group,
          &favorite, play_on_completion, play_modes.as_ref())?;
        Ok(())
      })
    })
  })
}
