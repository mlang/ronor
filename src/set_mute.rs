use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use super::{find_group_by_name, find_player_by_name, Result, ErrorKind};

pub const NAME: &str = "set-mute";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Set mute state for a group or player")
    .arg(Arg::with_name("UNMUTE").short("u").long("unmute"))
    .arg(Arg::with_name("GROUP").short("g").long("group")
         .takes_value(true).value_name("NAME"))
    .arg(Arg::with_name("PLAYER").short("p").long("player")
         .takes_value(true).value_name("NAME"))
    .group(ArgGroup::with_name("TARGET").args(&["GROUP", "PLAYER"]).required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let muted = !matches.is_present("UNMUTE");
    if matches.is_present("GROUP") {
      with_group!(sonos, matches, group, {
        Ok(sonos.set_group_mute(&group, muted)?)
      })
    } else {
      with_player!(sonos, matches, player, {
        Ok(sonos.set_player_mute(&player, muted)?)
      })
    }
  })
}
