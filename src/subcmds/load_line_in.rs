use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "load-line-in";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Change the given group to the line-in source of a specified player")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("PLAY")
        .short("p")
        .long("play")
        .help("Automatically start playback")
    )
    .arg(
      Arg::with_name("GROUP")
        .required(true)
        .help("Name of the group")
    )
    .arg(Arg::with_name("PLAYER").help("Name of the player"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let play_on_completion = matches.is_present("PLAY");
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let player = if matches.is_present("PLAYER") {
    Some(matches.player(&targets.players)?)
  } else {
    None
  };
  sonos.load_line_in(&group, player, play_on_completion)?;
  Ok(())
}
