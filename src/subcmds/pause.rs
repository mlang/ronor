use crate::{ArgMatchesExt, ErrorKind, Result};
use clap::{App, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "pause";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Pause playback for the given group")
    .arg(crate::household_arg())
    .arg(Arg::with_name("GROUP").help("Name of the group"))
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let group_name = matches.value_of("GROUP");
  let household = matches.household(sonos).await?;
  let mut found = false;
  for group in sonos.get_groups(&household).await?.groups.iter() {
    if group_name.map_or(true, |name| name == group.name) {
      found = true;
      sonos.pause(&group).await?;
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
