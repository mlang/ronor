use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("get-player-volume")
    .about("Get player volume")
    .arg(Arg::with_name("PLAYER"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for player in sonos.get_groups(&household)?.players.iter().filter(|player|
        matches.value_of("PLAYER").map_or(true, |name| name == player.name)
      ) {
        found = true;
        println!("{:?} => {:#?}", player.name, sonos.get_player_volume(&player)?);
      }
    }
    if matches.value_of("GROUP").is_some() && !found {
      println!("The specified player was not found");
      exit(1);
    }
    Ok(())
  })
}
