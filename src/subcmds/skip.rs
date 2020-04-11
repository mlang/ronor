use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "skip";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Go to next or previous track in the given group")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("NEXT")
        .short("n")
        .long("next-track")
        .help("Skip to next track")
    )
    .arg(
      Arg::with_name("PREVIOUS")
        .short("p")
        .long("previous-track")
        .help("Skip to previous track")
    )
    .group(ArgGroup::with_name("DIRECTION").args(&["NEXT", "PREVIOUS"]))
    .arg(Arg::with_name("GROUP").required(true))
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  let targets = sonos.get_groups(&household).await?;
  let group = matches.group(&targets.groups)?;
  if matches.is_present("NEXT") {
    sonos.skip_to_next_track(&group).await
  } else {
    sonos.skip_to_previous_track(&group).await
  }?;
  Ok(())
}
