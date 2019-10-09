use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::{find_group_by_name, find_player_by_name, Result};

pub const NAME: &'static str = "load-line-in";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Change the given group to the line-in source of a specified player")
    .arg(Arg::with_name("PLAY").short("p").long("play")
           .help("Automatically start playback"))
    .arg(Arg::with_name("GROUP").required(true)
           .help("Name of the group"))
    .arg(Arg::with_name("PLAYER")
           .help("Name of the player"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      match matches.value_of("PLAYER") {
        None => Ok(sonos.load_line_in(&group, None, matches.is_present("PLAY"))?),
        Some(player_name) => {
          match find_player_by_name(sonos, player_name)? {
            Some(player) => Ok(sonos.load_line_in(&group, Some(&player), matches.is_present("PLAY"))?),
            None => {
              println!("Player not found");
              exit(1)
            }
          }
        }
      }
    })
  })
}
