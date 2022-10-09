use crate::{ArgMatchesExt, ErrorKind, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "play";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Start playback for the given group")
    .arg(crate::household_arg())
    .arg(Arg::new("GROUP").help("Name of the group"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let group_name = matches.get_one::<String>("GROUP");
  let household = matches.household(sonos)?;
  let mut found = false;
  for group in sonos.get_groups(&household)?.groups.iter() {
    if group_name.map_or(true, |name| name == &group.name) {
      found = true;
      sonos.play(group)?;
    }
  }
  if !found {
    if let Some(group_name) = group_name {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
    return Err("No groups found".into());
  }
  Ok(())
}
