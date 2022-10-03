use crate::{ArgMatchesExt, ErrorKind, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::{Player, PlayerId, Sonos};

pub const NAME: &str = "modify-group";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Add or remove logical players to/from a group")
    .arg(crate::household_arg())
    .arg(
      Arg::new("GROUP")
        .required(true)
        .num_args(1)
        .help("The name of the group to modify")
    )
    .arg(
      Arg::new("ADD")
        .short('a')
        .long("add")
        .num_args(1..)
        .value_name("PLAYER_NAME")
        .value_parser(value_parser!(String))
        .help("Names of the logical players to add")
    )
    .arg(
      Arg::new("REMOVE")
        .short('r')
        .long("remove")
        .num_args(1..)
        .value_name("PLAYER_NAME")
        .help("Names of the logical players to remove")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let add = matches.get_many::<String>("ADD").map(|vals| vals.map(|x| x.to_string()).collect::<Vec<_>>()).unwrap_or_default();
  let remove = matches.get_many::<String>("REMOVE").map(|vals| vals.map(|x| x.to_string()).collect::<Vec<_>>()).unwrap_or_default();
  let player_ids_to_add = player_ids(add, &targets.players)?;
  let player_ids_to_remove = player_ids(remove, &targets.players)?;
  let modified_group =
    sonos.modify_group_members(&group, &player_ids_to_add, &player_ids_to_remove)?;
  println!("{} -> {}", group.name, modified_group.name);
  Ok(())
}

fn player_ids<'a>(
  names: Vec<String>,
  players: &'a [Player]
) -> Result<Vec<&'a PlayerId>> {
  let mut ids = Vec::new();
  for name in names.iter() {
    match players.iter().find(|p| &p.name == name) {
      None => return Err(ErrorKind::UnknownPlayer(name.to_string()).into()),
      Some(player) => ids.push(&player.id)
    }
  }
  Ok(ids)
}
