use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-home-theater-playback";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Signal a player to switch to its TV input (optical or HDMI)")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("PLAYER")
        .required(true)
        .help("Name of the player")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let player = matches.player(&targets.players)?;
  sonos.load_home_theater_playback(&player)?;
  Ok(())
}
