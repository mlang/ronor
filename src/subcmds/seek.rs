use crate::{ArgMatchesExt, Result, ResultExt};
use clap::{Command, Arg, ArgMatches};
use humantime::parse_duration;
use ronor::Sonos;

pub const NAME: &str = "seek";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Go to a specific position in the current track")
    .arg(crate::household_arg())
    .arg(
      Arg::new("FORWARD")
        .short('f')
        .long("forward")
        .conflicts_with("BACKWARD")
        .help("Seek forward relative to current position")
    )
    .arg(
      Arg::new("BACKWARD")
        .short('b')
        .long("backward")
        .conflicts_with("FORWARD")
        .help("Seek backward relative to current position")
    )
    .arg(
      Arg::new("TIME")
        .required(true)
        .help("Time specification (example: 2m3s)")
    )
    .arg(
      Arg::new("GROUP")
        .required(true)
        .help("Name of the group")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let backward = matches.contains_id("BACKWARD");
  let forward = matches.contains_id("BACKWARD");
  let relative = backward || forward;
  let time = matches.get_one::<String>("TIME").unwrap();
  let duration =
    parse_duration(time).chain_err(|| "Failed to parse time specification")?;
  if relative {
    sonos.seek_relative(
      &group,
      if backward {
        -(duration.as_millis() as i128)
      } else {
        duration.as_millis() as i128
      },
      None
    )
  } else {
    sonos.seek(&group, duration.as_millis(), None)
  }?;
  Ok(())
}
