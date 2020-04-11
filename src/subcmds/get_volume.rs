use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "get-volume";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get volume from a player or group")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("GROUP")
        .short("g")
        .long("group")
        .takes_value(true)
        .value_name("NAME")
    )
    .arg(
      Arg::with_name("PLAYER")
        .short("p")
        .long("player")
        .takes_value(true)
        .value_name("NAME")
    )
    .group(ArgGroup::with_name("TARGET").args(&["GROUP", "PLAYER"]))
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let mut found = false;
  let group_name = matches.value_of("GROUP");
  let player_name = matches.value_of("PLAYER");
  let household = matches.household(sonos).await?;
  let targets = sonos.get_groups(&household).await?;
  for player in targets.players.iter().filter(|player| {
    player_name.map_or(group_name.is_none(), |name| name == player.name)
  }) {
    found = true;
    println!(
      "{:?} => {:#?}",
      player.name,
      sonos.get_player_volume(&player).await?
    );
  }
  for group in targets
    .groups
    .iter()
    .filter(|group| group_name.map_or(player_name.is_none(), |name| name == group.name))
  {
    found = true;
    println!("{:?} => {:#?}", group.name, sonos.get_group_volume(&group).await?);
  }
  if !found {
    return Err("No group or player found".into());
  }
  Ok(())
}
