use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("play")
    .about("Start playback for the given group")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          sonos.play(&group)?;
        }
      }
    }
    if !found {
      println!("Group not found");
      exit(1);
    }
    Ok(())
  })
}

