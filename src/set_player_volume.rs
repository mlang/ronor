use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_player_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("set-player-volume")
    .about("Set player volume")
    .arg(Arg::with_name("RELATIVE").short("r").long("relative")
           .help("Indicates that the volume should be interpreted as relative"))
    .arg(Arg::with_name("PLAYER").required(true))
    .arg(Arg::with_name("VOLUME").required(true).help("Volume (0-100)"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      let volume = matches.value_of("VOLUME").unwrap();
      Ok(if matches.is_present("RELATIVE") {
        sonos.set_relative_player_volume(&player, volume.parse::<>()?)?
      } else {
        sonos.set_player_volume(&player, volume.parse::<>()?)?
      })
    })
  })
}

