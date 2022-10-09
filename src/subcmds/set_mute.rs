use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "set-mute";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Set mute state for a group or player")
    .arg(crate::household_arg())
    .arg(Arg::new("UNMUTE").short('u').long("unmute"))
    .arg(
      Arg::new("GROUP")
        .short('g')
        .long("group")
        .num_args(1)
        .value_name("NAME")
    )
    .arg(
      Arg::new("PLAYER")
        .short('p')
        .long("player")
        .num_args(1)
        .value_name("NAME")
    )
    .group(
      ArgGroup::new("TARGET")
        .args(&["GROUP", "PLAYER"])
        .required(true)
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let muted = !matches.contains_id("UNMUTE");
  if matches.contains_id("GROUP") {
    let group = matches.group(&targets.groups)?;
    sonos.set_group_mute(group, muted)
  } else {
    let player = matches.player(&targets.players)?;
    sonos.set_player_mute(player, muted)
  }?;
  Ok(())
}
