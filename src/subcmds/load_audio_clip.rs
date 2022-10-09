use crate::{ArgMatchesExt, Result};
use clap::{Command, Arg, ArgMatches, builder::PossibleValuesParser};
use ronor::Sonos;
use url::Url;

pub const NAME: &str = "load-audio-clip";
pub fn build() -> Command {
  Command::new(NAME)
    .about("Schedule an audio clip to play on a particular player")
    .arg(crate::household_arg())
    .arg(
      Arg::new("NAME")
        .default_value("ronor clip")
        .short('n')
        .long("name")
        .num_args(1)
    )
    .arg(
      Arg::new("APP_ID")
        .default_value("guru.blind")
        .value_name("STRING")
        .short('i')
        .long("app-id")
        .num_args(1)
    )
    .arg(
      Arg::new("CLIP_TYPE")
        .short('t')
        .long("type")
        .num_args(1)
        .value_parser(PossibleValuesParser::new(&["Chime", "Custom"]))
	.num_args(1)
    )
    .arg(
      Arg::new("PRIORITY")
        .short('p')
        .long("priority")
        .num_args(1)
        .value_parser(PossibleValuesParser::new(&["Low", "High"]))
	.num_args(1)
    )
    .arg(
      Arg::new("VOLUME")
        .short('v')
        .long("volume")
        .num_args(1)
        .help("Volume in percent (0-100)")
    )
    .arg(
      Arg::new("HTTP_AUTHORIZATION")
        .short('a')
        .long("http-authorization")
        .num_args(1)
        .value_name("STRING")
        .help("HTTP Authorization string")
    )
    .arg(
      Arg::new("PLAYER")
        .required(true)
        .help("Name of the player")
    )
    .arg(
      Arg::new("URL")
        .required(true)
        .help("Location of the audio clip")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household = matches.household(sonos)?;
  let targets = sonos.get_groups(&household)?;
  let player = matches.player(&targets.players)?;
  let url = matches.get_one::<Url>("URL").unwrap();
  if url.has_host() {
    let http_auth = matches.get_one::<String>("HTTP_AUTHORIZATION");
    sonos.load_audio_clip(
      player,
      matches.get_one::<String>("APP_ID").unwrap(),
      matches.get_one::<String>("NAME").unwrap(),
      match matches.get_one::<String>("CLIP_TYPE") {
        Some(s) => Some(s.parse()?),
        None => None
      },
      match matches.get_one::<String>("PRIORITY") {
        Some(s) => Some(s.parse()?),
        None => None
      },
      match matches.get_one::<String>("VOLUME") {
        Some(s) => Some(s.parse()?),
        None => None
      },
      http_auth.map(|a| a.as_str()),
      Some(url)
    )?;
  } else {
    return Err(
      "The URL you provided does not look like Sonos will be able to reach it".into()
    );
  }
  Ok(())
}
