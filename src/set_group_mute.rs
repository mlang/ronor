use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("set-group-mute")
    .about("Set group mute state")
    .arg(Arg::with_name("UNMUTE").short("u").long("unmute"))
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(sonos.set_group_mute(&group, !matches.is_present("UNMUTE"))?)
    })
  })
}
