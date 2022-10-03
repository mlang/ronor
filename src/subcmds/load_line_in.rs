use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-line-in";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Change the given group to the line-in source of a specified player")
    .arg(crate::household_arg())
    .arg(
      Arg::new("PLAY")
        .short('p')
        .long("play")
        .help("Automatically start playback")
    )
    .arg(
      Arg::new("GROUP")
        .required(true)
        .help("Name of the group")
    )
    .arg(Arg::new("PLAYER").help("Name of the player"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let play_on_completion = matches.contains_id("PLAY");
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let player = if matches.contains_id("PLAYER") {
    Some(matches.player(&targets.players)?)
  } else {
    None
  };
  sonos.load_line_in(&group, player, play_on_completion)?;
  Ok(())
}
