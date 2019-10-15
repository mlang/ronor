use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{Result, ErrorKind, ArgMatchesExt};

pub const NAME: &str = "play";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Start playback for the given group")
    .arg(super::household_arg())
    .arg(Arg::with_name("GROUP")
         .help("Name of the group"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let group_name = matches.value_of("GROUP");
    let household = matches.household(sonos)?;
    let mut found = false;
    for group in sonos.get_groups(&household)?.groups.iter() {
      if group_name.map_or(true, |name| name == group.name) {
        found = true;
        sonos.play(&group)?;
      }
    }
    if !found {
      if group_name.is_some() {
        return Err(ErrorKind::UnknownGroup(group_name.unwrap().to_string()).into());
      }
      return Err("No groups found".into());
    }
    Ok(())
  })
}
