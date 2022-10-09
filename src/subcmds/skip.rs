use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "skip";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Go to next or previous track in the given group")
    .arg(crate::household_arg())
    .arg(
      Arg::new("NEXT")
        .short('n')
        .long("next-track")
        .help("Skip to next track")
    )
    .arg(
      Arg::new("PREVIOUS")
        .short('p')
        .long("previous-track")
        .help("Skip to previous track")
    )
    .group(ArgGroup::new("DIRECTION").args(&["NEXT", "PREVIOUS"]))
    .arg(Arg::new("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  if matches.contains_id("NEXT") {
    sonos.skip_to_next_track(group)
  } else {
    sonos.skip_to_previous_track(group)
  }?;
  Ok(())
}
