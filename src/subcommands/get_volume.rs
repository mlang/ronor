use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use crate::{Result, ArgMatchesExt};

pub const NAME: &str = "get-volume";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get volume from a player or group")
    .arg(crate::household_arg())
    .arg(Arg::with_name("GROUP").short("g").long("group").takes_value(true).value_name("NAME"))
    .arg(Arg::with_name("PLAYER").short("p").long("player").takes_value(true).value_name("NAME"))
    .group(ArgGroup::with_name("TARGET").args(&["GROUP", "PLAYER"]))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  let group_name = matches.value_of("GROUP");
  let player_name = matches.value_of("PLAYER");
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  for player in targets.players.iter().filter(|player|
    player_name.map_or(group_name.is_none(), |name| name == player.name)
  ) {
    found = true;
    println!("{:?} => {:#?}", player.name, sonos.get_player_volume(&player)?);
  }
  for group in targets.groups.iter().filter(|group|
    group_name.map_or(player_name.is_none(), |name| name == group.name)
  ) {
    found = true;
    println!("{:?} => {:#?}", group.name, sonos.get_group_volume(&group)?);
  }
  if !found {
    return Err("No group or player found".into());
  }
  Ok(())
}
