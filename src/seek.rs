use clap::{Arg, ArgMatches, App};
use humantime::parse_duration;
use ronor::Sonos;
use super::{find_group_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("seek")
    .about("Go to a specific position in the current track")
    .arg(Arg::with_name("PLUS").short("p").long("plus").conflicts_with("MINUS")
           .help("Seek relative to current position"))
    .arg(Arg::with_name("MINUS").short("m").long("minus").conflicts_with("PLUS")
           .help("Seek backwards relative to current position"))
    .arg(Arg::with_name("GROUP").required(true))
    .arg(Arg::with_name("TIME").required(true)
           .help("Position in milliseconds"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      let time = matches.value_of("TIME").unwrap();
      match parse_duration(time) {
        Ok(time) => {
          if matches.is_present("MINUS") || matches.is_present("PLUS") {
            Ok(sonos.seek_relative(&group,
                if matches.is_present("MINUS") {
                  -(time.as_millis() as i128)
                } else {
                  time.as_millis() as i128
                }, None)?)
          } else {
            Ok(sonos.seek(&group, time.as_millis(), None)?)
          }
        },
        Err(_) => Err("Failed to parse time".into())
      }
    })
  })
}
