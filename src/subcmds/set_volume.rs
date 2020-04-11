use crate::{ArgMatchesExt, Result};
use clap::{App, Arg, ArgGroup, ArgMatches};
use ronor::Sonos;

pub const NAME: &str = "set-volume";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Set volume for a group or player")
    .arg(crate::household_arg())
    .arg(
      Arg::with_name("INCREMENT")
        .short("i")
        .long("increment")
        .help("Increase volume")
    )
    .arg(
      Arg::with_name("DECREMENT")
        .short("d")
        .long("decrement")
        .help("Decrease volume")
    )
    .group(
      ArgGroup::with_name("RELATIVE")
        .args(&["INCREMENT", "DECREMENT"])
    )
    .arg(
      Arg::with_name("GROUP")
        .short("g")
        .long("group")
        .takes_value(true)
        .value_name("NAME")
    )
    .arg(
      Arg::with_name("PLAYER")
        .short("p")
        .long("player")
        .takes_value(true)
        .value_name("NAME")
    )
    .group(
      ArgGroup::with_name("TARGET")
        .args(&["GROUP", "PLAYER"])
        .required(true)
    )
    .arg(
      Arg::with_name("VOLUME")
        .required(true)
        .help("Volume in percent")
    )
}

pub async fn run(sonos: &mut Sonos, matches: &ArgMatches<'_>) -> Result<()> {
  let household = matches.household(sonos).await?;
  let targets = sonos.get_groups(&household).await?;
  let increment = matches.is_present("INCREMENT");
  let decrement = matches.is_present("DECREMENT");
  let volume = matches.value_of("VOLUME").unwrap();
  if matches.is_present("GROUP") {
    let group = matches.group(&targets.groups)?;
    if increment {
      sonos.set_relative_group_volume(&group, volume.parse()?).await
    } else if decrement {
      sonos.set_relative_group_volume(&group, -volume.parse()?).await
    } else {
      sonos.set_group_volume(&group, volume.parse()?).await
    }
  } else {
    let player = matches.player(&targets.players)?;
    if increment {
      sonos.set_relative_player_volume(&player, volume.parse()?).await
    } else if decrement {
      sonos.set_relative_player_volume(&player, -volume.parse()?).await
    } else {
      sonos.set_player_volume(&player, volume.parse()?).await
    }
  }?;
  Ok(())
}
