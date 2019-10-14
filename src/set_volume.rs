use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use super::{find_group_by_name, find_player_by_name, Result, ErrorKind};

pub const NAME: &str = "set-volume";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Set volume for a group or player")
    .arg(Arg::with_name("RELATIVE").short("r").long("relative")
           .help("Indicates that the volume should be interpreted as relative"))
    .arg(Arg::with_name("GROUP")
         .short("g").long("group")
         .takes_value(true).value_name("NAME"))
    .arg(Arg::with_name("PLAYER")
         .short("p").long("player")
         .takes_value(true).value_name("NAME"))
    .group(ArgGroup::with_name("TARGET").args(&["GROUP", "PLAYER"]).required(true))
    .arg(Arg::with_name("VOLUME").required(true)
           .help("Volume in percent"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let relative = matches.is_present("RELATIVE");
    let volume = matches.value_of("VOLUME").unwrap();
    if matches.is_present("GROUP") {
      with_group!(sonos, matches, group, {
        if relative {
          Ok(sonos.set_relative_group_volume(&group, volume.parse::<>()?)?)
        } else {
          Ok(sonos.set_group_volume(&group, volume.parse::<>()?)?)
        }
      })
    } else {
      with_player!(sonos, matches, player, {
        if relative {
          Ok(sonos.set_relative_player_volume(&player, volume.parse::<>()?)?)
        } else {
          Ok(sonos.set_player_volume(&player, volume.parse::<>()?)?)
        }
      })
    }
  })
}
