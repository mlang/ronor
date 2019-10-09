use clap::{Arg, ArgMatches, App};
use ronor::{Sonos, PlayModes};
use super::{find_group_by_name, find_playlist_by_name, Result};

pub const NAME: &'static str = "load-playlist";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Load the specified playlist in a group")
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
    with_playlist!(sonos, matches, playlist, {
      with_group!(sonos, matches, group, {
        let repeat = matches.is_present("REPEAT");
        let repeat_one = matches.is_present("REPEAT_ONE");
        let crossfade = matches.is_present("CROSSFADE");
        let shuffle = matches.is_present("SHUFFLE");
        let play_modes = PlayModes { repeat, repeat_one, crossfade, shuffle };
        Ok(sonos.load_playlist(&group, &playlist,
            matches.is_present("PLAY"),
            if repeat || repeat_one || crossfade || shuffle {
              Some(&play_modes)
            } else {
              None
            }
          )?
        )
      })
    })
  })
}
