use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::{Result, ArgMatchesExt};

pub const NAME: &str = "load-favorite";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Load the specified favorite in a group")
    .arg(super::household_arg())
    .arg(Arg::with_name("PLAY").short("p").long("play")
           .help("Automatically start playback"))
    .args(&super::play_modes_args())
    .arg(Arg::with_name("FAVORITE").required(true))
    .arg(Arg::with_name("GROUP").required(true))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let favorite = matches.favorite(sonos, &household)?;
  let targets = sonos.get_groups(&household)?;
  let group = matches.group(&targets.groups)?;
  let play_on_completion = matches.is_present("PLAY");
  sonos.load_favorite(&group,
    &favorite, play_on_completion, matches.play_modes().as_ref())?;
  Ok(())
}
