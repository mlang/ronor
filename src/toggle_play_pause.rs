use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::{find_player_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("toggle-play-pause")
    .about("Toggle the playback state of the given group")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          sonos.toggle_play_pause(&group)?;
        }
      }
    }
    if matches.value_of("GROUP").is_some() && !found {
      println!("Group not found");
      exit(1);
    }
    Ok(())
  })
}

