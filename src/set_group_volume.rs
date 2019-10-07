use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("set-group-volume")
    .about("Set group volume")
    .arg(Arg::with_name("RELATIVE").short("r").long("relative")
           .help("Indicates that the volume should be interpreted as relative"))
    .arg(Arg::with_name("GROUP").required(true))
    .arg(Arg::with_name("VOLUME").required(true)
           .help("Volume in percent"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      let volume = matches.value_of("VOLUME").unwrap();
      if matches.is_present("RELATIVE") {
        Ok(sonos.set_relative_group_volume(&group, volume.parse::<>()?)?)
      } else {
        Ok(sonos.set_group_volume(&group, volume.parse::<>()?)?)
      }
    })
  })
}
