#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{Arg, ArgMatches, App, AppSettings};
use ronor::{Sonos, Favorite, Group, Household, Player, Playlist, PlayModes};
use std::convert::TryFrom;
use xdg::BaseDirectories;

error_chain! {
  errors {
    UnknownGroup(name: String) {
      description("Group not found")
      display("No such group named '{}'", name)
    }
    UnknownPlayer(name: String) {
      description("Player not found")
      display("No such player named '{}'", name)
    }
  }
  links {
    API(ronor::Error, ronor::ErrorKind);
  }
  foreign_links {
    IO(std::io::Error);
    XDG(xdg::BaseDirectoriesError);
    ReadLine(rustyline::error::ReadlineError);
    ParseInt(std::num::ParseIntError);
    UrlParse(url::ParseError);
    Clap(clap::Error);
    Duration(humantime::DurationError);
  }
}

macro_rules! subcmds {
  ($($mod:ident),*) => {
    mod subcommands {
      $(pub(crate) mod $mod;)*
    }
    fn build_subcommands() -> Vec<App<'static, 'static>> {
      vec![$(subcommands::$mod::build()),*]
    }
    fn run_subcommand(sonos: &mut Sonos, name: &str, matches: Option<&ArgMatches>) -> Result<()> {
      match (name, matches) {
        $((subcommands::$mod::NAME, Some(matches)) => subcommands::$mod::run(sonos, matches),)*
        _ => unimplemented!()
      }
    }
  }
}

subcmds!(init, login, get_favorites, get_playlist, get_playlists,
  get_volume, inventory, load_audio_clip, load_favorite, load_home_theater_playback,
  load_line_in, load_playlist, modify_group, now_playing, pause, play, seek,
  set_mute, set_volume, skip, speak, toggle_play_pause
);

fn build_cli() -> App<'static, 'static> {
  App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .setting(AppSettings::ArgRequiredElseHelp)
    .subcommands(build_subcommands())
    .subcommand(App::new("get-groups").setting(AppSettings::Hidden)
      .about("Get list of groups"))
    .subcommand(App::new("get-players").setting(AppSettings::Hidden)
      .about("Get list of players"))
    .subcommand(App::new("get-playback-status").setting(AppSettings::Hidden)
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP")))
    .subcommand(App::new("get-metadata-status").setting(AppSettings::Hidden)
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP")))
    .subcommand(App::new("completions").setting(AppSettings::Hidden)
      .about("Generates completion scripts for your shell")
      .arg(Arg::with_name("SHELL")
             .required(true)
             .possible_values(&["bash", "fish", "zsh"])
             .help("The shell to generate the script for")))
}

quick_main!(run);

fn run() -> Result<()> {
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  //let players = player_names(&mut sonos)?;
  //let players: Vec<&str> = players.iter().map(|x| x.as_str()).collect();
  match build_cli().get_matches().subcommand() {
    ("completions", Some(matches)) => {
      let shell = matches.value_of("SHELL").unwrap();
      build_cli().gen_completions_to(
        "ronor",
        shell.parse::<>().unwrap(),
        &mut std::io::stdout()
      );
      Ok(())
    },
    ("get-playback-status", Some(matches)) =>
      get_playback_status(&mut sonos, matches),
    ("get-groups", Some(matches)) =>      get_groups(&mut sonos, matches),
    ("get-metadata-status", Some(matches)) =>
      get_metadata_status(&mut sonos, matches),
    ("get-players", Some(matches)) =>     get_players(&mut sonos, matches),
    (cmd, matches) => run_subcommand(&mut sonos, cmd, matches)
  }
}

fn get_playback_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
      matches.value_of("GROUP").map_or(true, |name| name == group.name)
    ) {
      found = true;
      println!("{:?} => {:#?}", group.name, sonos.get_playback_status(&group)?);
    }
  }
  if !found {
    if let Some(group_name) = matches.value_of("GROUP") {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
  }
  Ok(())
}

fn get_metadata_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
      matches.value_of("GROUP").map_or(true, |name| name == group.name)
    ) {
      found = true;
      println!("{:?} => {:#?}", group.name, sonos.get_metadata_status(&group)?);
    }
  }
  if !found {
    if let Some(group_name) = matches.value_of("GROUP") {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
  }
  Ok(())
}

fn get_groups(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(&household)?.groups.iter() {
      println!("{}", group.name);
    }
  }
  Ok(())
}

fn get_players(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  for household in sonos.get_households()?.iter() {
    for player in sonos.get_groups(&household)?.players.iter() {
      println!("{}", player.name);
    }
  }
  Ok(())
}

fn household_arg() -> Arg<'static, 'static> {
  Arg::with_name("HOUSEHOLD")
    .long("household").takes_value(true).value_name("INDEX")
    .help("Optional 0-based household index")
}

fn play_modes_args() -> Vec<Arg<'static, 'static>> {
  vec![Arg::with_name("REPEAT").short("r").long("repeat"),
       Arg::with_name("REPEAT_ONE").short("o").long("repeat-one"),
       Arg::with_name("CROSSFADE").short("c").long("crossfade")
         .help("Do crossfade between tracks"),
       Arg::with_name("SHUFFLE").short("s").long("shuffle")
         .help("Shuffle the tracks")
  ]
}

trait ArgMatchesExt {
  fn household(self: &Self, sonos: &mut Sonos) -> Result<Household>;
  fn favorite(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Favorite>;
  fn group<'a>(self: &Self, groups: &'a [Group]) -> Result<&'a Group>;
  fn player<'a>(self: &Self, players: &'a [Player]) -> Result<&'a Player>;
  fn playlist(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Playlist>;
  fn play_modes(self: &Self) -> Option<PlayModes>;
}

impl ArgMatchesExt for ArgMatches<'_> {
  fn household(self: &Self, sonos: &mut Sonos) -> Result<Household> {
    let households = sonos.get_households()?;
    match households.len() {
      0 => Err("No households found".into()),
      1 => Ok(households.into_iter().next().unwrap()),
      _ => match self.value_of("HOUSEHOLD") {
        None => Err("Multiple households found".into()),
        Some(index) => {
          let index = index.parse::<usize>().chain_err(|| "Invalid household index")?;
          for (n, household) in households.into_iter().enumerate() {
            if n == index {
              return Ok(household)
            }
          }
          Err("Household out of range".into())
        }
      }
    }
  }
  fn favorite(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Favorite> {
    let favorite_name = self.value_of("FAVORITE").unwrap();
    for favorite in sonos.get_favorites(household)?.items.into_iter() {
      if favorite.name == favorite_name {
        return Ok(favorite)
      }
    }
    Err("Playlist not found".into())
  }
  fn playlist(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Playlist> {
    let playlist_name = self.value_of("PLAYLIST").unwrap();
    for playlist in sonos.get_playlists(household)?.playlists.into_iter() {
      if playlist.name == playlist_name {
        return Ok(playlist)
      }
    }
    Err("Playlist not found".into())
  }
  fn group<'a>(self: &Self, groups: &'a [Group]) -> Result<&'a Group> {
    let group_name = self.value_of("GROUP").unwrap();
    for group in groups.iter() {
      if group.name == group_name {
        return Ok(&group);
      }
    }
    Err(ErrorKind::UnknownGroup(group_name.to_string()).into())
  }
  fn player<'a>(self: &Self, players: &'a [Player]) -> Result<&'a Player> {
    let player_name = self.value_of("PLAYER").unwrap();
    for player in players.iter() {
      if player.name == player_name {
        return Ok(&player);
      }
    }
    Err(ErrorKind::UnknownPlayer(player_name.to_string()).into())
  }
  fn play_modes(self: &Self) -> Option<PlayModes> {
    let repeat = self.is_present("REPEAT");
    let repeat_one = self.is_present("REPEAT_ONE");
    let crossfade = self.is_present("CROSSFADE");
    let shuffle = self.is_present("SHUFFLE");
    if repeat || repeat_one || crossfade || shuffle {
      Some(PlayModes { repeat, repeat_one, crossfade, shuffle })
    } else {
      None
    }
  }
}
