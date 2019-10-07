use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{find_player_by_name, Result};

pub fn build() -> App<'static, 'static> {
  App::new("load-home-theater-playback")
    .about("Signal a player to switch to its TV input (optical or HDMI)")
    .arg(Arg::with_name("PLAYER").required(true)
           .help("Name of the player"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      Ok(sonos.load_home_theater_playback(&player)?)
    })
  })
}
