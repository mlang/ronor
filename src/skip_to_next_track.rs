use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub const NAME: &'static str = "skip-to-next-track";

pub fn build() -> App<'static, 'static> {
  App::new(NAME).visible_alias("next")
    .about("Go to next track in the given group")
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(sonos.skip_to_next_track(&group)?)
    })
  })
}
