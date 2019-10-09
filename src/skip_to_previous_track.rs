use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub const NAME: &'static str = "skip-to-previous-track";

pub fn build() -> App<'static, 'static> {
  App::new(NAME).visible_alias("prev")
    .about("Go to previous track in the given group")
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(sonos.skip_to_previous_track(&group)?)
    })
  })
}
