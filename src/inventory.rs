use clap::{Arg, ArgMatches, ArgGroup, App};
use ronor::Sonos;
use std::process::exit;
use super::Result;

pub const NAME: &str = "inventory";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Describes available households, groups and logical players")
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      println!("Household: {}", household.id);
      let targets = sonos.get_groups(&household)?;
      for group in targets.groups.iter() {
        print!("{}", group.name);
        let mut player_ids = group.player_ids.iter();
        if let Some(player_id) = player_ids.next() {
          if let Some(player) = targets.players.iter().find(|player|
            &player.id == player_id
          ) {
            print!(" = {}", player.name);
            for player_id in player_ids {
              if let Some(player) = targets.players.iter().find(|player|
                &player.id == player_id
              ) {
                print!(" + {}", player.name);
              }
            }
          }
        }
        println!();
      }
    }
    Ok(())
  })
}
