use crate::{ArgMatchesExt, ErrorKind, Result};
use clap::{App, Arg, ArgMatches};
use ronor::{Player, PlayerId, Sonos};

pub const NAME: &str = "modify-group";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Add or remove logical players to/from a group")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("GROUP")
        .required(true)
        .takes_value(true)
        .help("The name of the group to modify")
    )
    .arg(
      Arg::with_name("ADD")
        .short("a")
        .long("add")
        .takes_value(true)
        .value_name("PLAYER_NAME")
        .multiple(true)
        .help("Names of the logical players to add")
    )
    .arg(
      Arg::with_name("REMOVE")
        .short("r")
        .long("remove")
        .takes_value(true)
        .value_name("PLAYER_NAME")
        .multiple(true)
        .help("Names of the logical players to remove")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let player_ids_to_add = player_ids(matches.values_of("ADD"), &targets.players)?;
  let player_ids_to_remove = player_ids(matches.values_of("REMOVE"), &targets.players)?;
  let modified_group =
    sonos.modify_group_members(&group, &player_ids_to_add, &player_ids_to_remove)?;
  println!("{} -> {}", group.name, modified_group.name);
  Ok(())
}

fn player_ids<'a, 'b, I: Iterator<Item = &'a str>>(
  names: Option<I>,
  players: &'b [Player]
) -> Result<Vec<&'b PlayerId>> {
  let mut ids = Vec::new();
  if let Some(names) = names {
    for name in names {
      match players.iter().find(|p| p.name == name) {
        None => return Err(ErrorKind::UnknownPlayer(name.to_string()).into()),
        Some(player) => ids.push(&player.id)
      }
    }
  }
  Ok(ids)
}
