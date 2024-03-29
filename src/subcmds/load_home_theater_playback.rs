use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-home-theater-playback";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Signal a player to switch to its TV input (optical or HDMI)")
    .arg(crate::household_arg())
    .arg(
      Arg::new("PLAYER")
        .required(true)
        .help("Name of the player")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let player = matches.player(&targets.players)?;
  sonos.load_home_theater_playback(player)?;
  Ok(())
}
