use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgAction, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "set-volume";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Set volume for a group or player")
    .arg(crate::household_arg())
    .arg(
      Arg::new("INCREMENT")
        .short('i')
        .long("increment")
        .action(ArgAction::SetTrue)
        .help("Increase volume")
    )
    .arg(
      Arg::new("DECREMENT")
        .short('d')
        .long("decrement")
        .action(ArgAction::SetTrue)
        .help("Decrease volume")
    )
    .group(
      ArgGroup::new("RELATIVE")
        .args(&["INCREMENT", "DECREMENT"])
    )
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
    .arg(
      Arg::new("VOLUME")
        .required(true)
        .help("Volume in percent")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let increment = matches.contains_id("INCREMENT");
  let decrement = matches.contains_id("DECREMENT");
  let volume = matches.get_one::<String>("VOLUME").unwrap();
  if matches.contains_id("GROUP") {
    let group = matches.group(&targets.groups)?;
    if increment {
      sonos.set_relative_group_volume(group, volume.parse()?)
    } else if decrement {
      sonos.set_relative_group_volume(group, -volume.parse()?)
    } else {
      sonos.set_group_volume(group, volume.parse()?)
    }
  } else {
    let player = matches.player(&targets.players)?;
    if increment {
      sonos.set_relative_player_volume(player, volume.parse()?)
    } else if decrement {
      sonos.set_relative_player_volume(player, -volume.parse()?)
    } else {
      sonos.set_player_volume(player, volume.parse()?)
    }
  }?;
  Ok(())
}
