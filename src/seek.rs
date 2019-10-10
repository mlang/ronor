use clap::{Arg, ArgMatches, App};
use humantime::parse_duration;
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub const NAME: &str = "seek";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Go to a specific position in the current track")
    .arg(Arg::with_name("FORWARD").short("f").long("forward").conflicts_with("BACKWARD")
           .help("Seek forward relative to current position"))
    .arg(Arg::with_name("BACKWARD").short("b").long("backward").conflicts_with("FORWARD")
           .help("Seek backward relative to current position"))
    .arg(Arg::with_name("TIME").required(true)
           .help("Time specification (example: 2m3s)"))
    .arg(Arg::with_name("GROUP").required(true)
           .help("Name of the group"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      let backward = matches.is_present("BACKWARD");
      let forward = matches.is_present("BACKWARD");
      let relative = backward || forward;
      let time = matches.value_of("TIME").unwrap();
      match parse_duration(time) {
        Ok(duration) => {
          if relative {
            Ok(sonos.seek_relative(&group,
                if backward {
                  -(duration.as_millis() as i128)
                } else {
                  duration.as_millis() as i128
                }, None)?)
          } else {
            Ok(sonos.seek(&group, duration.as_millis(), None)?)
          }
        },
        Err(_) => Err("Failed to parse time".into())
      }
    })
  })
}
