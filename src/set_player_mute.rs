use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_player_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("set-player-mute")
    .about("Set player mute state")
    .arg(Arg::with_name("UNMUTE").short("u").long("unmute"))
    .arg(Arg::with_name("PLAYER").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, group, {
      Ok(sonos.set_player_mute(&group, !matches.is_present("UNMUTE"))?)
    })
  })
}
