use crate::{ArgMatchesExt, Result, ResultExt};
use clap::{App, Arg, ArgMatches};
use humantime::parse_duration;
use ronor::Sonos;

pub const NAME: &str = "seek";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Go to a specific position in the current track")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("FORWARD")
        .short("f")
        .long("forward")
        .conflicts_with("BACKWARD")
        .help("Seek forward relative to current position")
    )
    .arg(
      Arg::with_name("BACKWARD")
        .short("b")
        .long("backward")
        .conflicts_with("FORWARD")
        .help("Seek backward relative to current position")
    )
    .arg(
      Arg::with_name("TIME")
        .required(true)
        .help("Time specification (example: 2m3s)")
    )
    .arg(
      Arg::with_name("GROUP")
        .required(true)
        .help("Name of the group")
    )
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  let targets = sonos.get_groups(&household).await?;
  let group = matches.group(&targets.groups)?;
  let backward = matches.is_present("BACKWARD");
  let forward = matches.is_present("BACKWARD");
  let relative = backward || forward;
  let time = matches.value_of("TIME").unwrap();
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
    ).await
  } else {
    sonos.seek(&group, duration.as_millis(), None).await
  }?;
  Ok(())
}
