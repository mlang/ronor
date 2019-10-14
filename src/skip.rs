use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use super::{find_group_by_name, Result, ErrorKind};

pub const NAME: &str = "skip";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Go to next or previous track in the given group")
    .arg(Arg::with_name("NEXT").short("n").long("next-track")
         .help("Skip to next track"))
    .arg(Arg::with_name("PREVIOUS").short("p").long("previous-track")
         .help("Skip to previous track"))
    .group(ArgGroup::with_name("DIRECTION").args(&["NEXT", "PREVIOUS"]))
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      if matches.is_present("NEXT") {
        Ok(sonos.skip_to_next_track(&group)?)
      } else {
        Ok(sonos.skip_to_previous_track(&group)?)
      }
    })
  })
}
