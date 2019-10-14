use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_group_by_name, find_player_by_name, Result, ErrorKind};

pub const NAME: &str = "load-line-in";

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
  let play_on_completion = matches.is_present("PLAY");
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      let player = match matches.value_of("PLAYER") {
        Some(player_name) => {
          match find_player_by_name(sonos, player_name)? {
            Some(player) => Some(player),
            None => return Err(ErrorKind::UnknownPlayer(player_name.to_string()).into()),
          }
        }
        None => None,
      };
      Ok(sonos.load_line_in(&group, player.as_ref(), play_on_completion)?)
    })
  })
}
