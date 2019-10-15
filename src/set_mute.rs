use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use super::{Result, ArgMatchesExt};

pub const NAME: &str = "set-mute";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Set mute state for a group or player")
    .arg(super::household_arg())
    .arg(Arg::with_name("UNMUTE").short("u").long("unmute"))
    .arg(Arg::with_name("GROUP").short("g").long("group")
         .takes_value(true).value_name("NAME"))
    .arg(Arg::with_name("PLAYER").short("p").long("player")
         .takes_value(true).value_name("NAME"))
    .group(ArgGroup::with_name("TARGET").args(&["GROUP", "PLAYER"]).required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let household = matches.household(sonos)?;
    let targets = sonos.get_groups(&household)?;
    let muted = !matches.is_present("UNMUTE");
    if matches.is_present("GROUP") {
      let group = matches.group(&targets.groups)?;
      Ok(sonos.set_group_mute(&group, muted)?)
    } else {
      let player = matches.player(&targets.players)?;
      Ok(sonos.set_player_mute(&player, muted)?)
    }
  })
}
