use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-volume";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Get volume from a player or group")
    .arg(crate::household_arg())
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
    .group(ArgGroup::new("TARGET").args(&["GROUP", "PLAYER"]))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  let group_name = matches.get_one::<String>("GROUP");
  let player_name = matches.get_one::<String>("PLAYER");
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  for player in targets.players.iter().filter(|player| {
    player_name.map_or(group_name.is_none(), |name| name == &player.name)
  }) {
    found = true;
    println!(
      "{:?} => {:#?}",
      player.name,
      sonos.get_player_volume(player)?
    );
  }
  for group in targets
    .groups
    .iter()
    .filter(|group| group_name.map_or(player_name.is_none(), |name| name == &group.name))
  {
    found = true;
    println!("{:?} => {:#?}", group.name, sonos.get_group_volume(group)?);
  }
  if !found {
    return Err("No group or player found".into());
  }
  Ok(())
}
