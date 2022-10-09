#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{builder::PossibleValuesParser, Arg, ArgMatches, Command};
use ronor::{Favorite, Group, Household, PlayModes, Player, Playlist, Sonos};
use std::convert::TryFrom;
use xdg::BaseDirectories;

error_chain! {
  errors {
    UnknownFavorite(name: String) {
      description("Favorite not found")
      display("No such favorite named '{}'", name)
    }
    UnknownGroup(name: String) {
      description("Group not found")
      display("No such group named '{}'", name)
    }
    UnknownPlayer(name: String) {
      description("Player not found")
      display("No such player named '{}'", name)
    }
    UnknownPlaylist(name: String) {
      description("Playlist not found")
      display("No such playlist named '{}'", name)
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
    Reqwest(reqwest::Error);
  }
}

trait CLI {
  fn run_subcmd(&mut self, name: &str, matches: &ArgMatches) -> Result<()>;
}

macro_rules! subcmds {
  (mod $subcmds:ident { $(mod $mod:ident;)* }) => {
    mod $subcmds {
      $(pub(crate) mod $mod;)*
    }
    fn build_subcmds() -> Vec<Command> {
      vec![$($subcmds::$mod::build()),*]
    }
    impl CLI for Sonos {
      fn run_subcmd(&mut self, name: &str, matches: &ArgMatches) -> Result<()> {
        match name {
          $($subcmds::$mod::NAME => $subcmds::$mod::run(self, matches),)*
          _ => unimplemented!()
        }
      }
    }
  }
}

subcmds!(
  mod subcmds {
    mod get_favorites;
    mod get_playlist;
    mod get_playlists;
    mod get_volume;
    mod init;
    mod inventory;
    mod load_audio_clip;
    mod load_favorite;
    mod load_home_theater_playback;
    mod load_line_in;
    mod load_playlist;
    mod login;
    mod modify_group;
    mod now_playing;
    mod pause;
    mod play;
    mod seek;
    mod set_mute;
    mod set_volume;
    mod skip;
    mod speak;
    mod toggle_play_pause;
  }
);

fn build() -> Command {
  Command::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about(crate_description!())
    .arg_required_else_help(true)
    .subcommands(build_subcmds())
    .subcommand(
      Command::new("get-groups")
        .hide(true)
        .about("Get list of groups"),
    )
    .subcommand(
      Command::new("get-players")
        .hide(true)
        .about("Get list of players"),
    )
    .subcommand(
      Command::new("get-playback-status")
        .hide(true)
        .about("Get playback status (DEBUG)")
        .arg(Arg::new("GROUP")),
    )
    .subcommand(
      Command::new("get-metadata-status")
        .hide(true)
        .about("Get playback status (DEBUG)")
        .arg(Arg::new("GROUP")),
    )
    .subcommand(
      Command::new("completions")
        .hide(true)
        .about("Generates completion scripts for your shell")
        .arg(
          Arg::new("SHELL")
            .value_parser(PossibleValuesParser::new(&["bash", "fish", "zsh"]))
            .num_args(1)
            .required(true)
            .help("The shell to generate the script for"),
        ),
    )
}

quick_main!(run);

fn run() -> Result<()> {
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  //let players = player_names(&mut sonos)?;
  //let players: Vec<&str> = players.iter().map(|x| x.as_str()).collect();
  match build().get_matches().subcommand() {
    Some(("get-playback-status", matches)) => get_playback_status(&mut sonos, matches),
    Some(("get-groups", matches)) => get_groups(&mut sonos, matches),
    Some(("get-metadata-status", matches)) => get_metadata_status(&mut sonos, matches),
    Some(("get-players", matches)) => get_players(&mut sonos, matches),
    Some((cmd, matches)) => sonos.run_subcmd(cmd, matches),
    _ => unreachable!(),
  }
}

fn get_playback_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(household)?.groups.iter().filter(|group| {
      matches
        .get_one::<String>("GROUP")
        .map_or(true, |name| name == &group.name)
    }) {
      found = true;
      println!(
        "{:?} => {:#?}",
        group.name,
        sonos.get_playback_status(group)?
      );
    }
  }
  if !found {
    if let Some(group_name) = matches.get_one::<String>("GROUP") {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
  }
  Ok(())
}

fn get_metadata_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let mut found = false;
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(household)?.groups.iter().filter(|group| {
      matches
        .get_one::<String>("GROUP")
        .map_or(true, |name| name == &group.name)
    }) {
      found = true;
      println!(
        "{:?} => {:#?}",
        group.name,
        sonos.get_metadata_status(group)?
      );
    }
  }
  if !found {
    if let Some(group_name) = matches.get_one::<String>("GROUP") {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
  }
  Ok(())
}

fn get_groups(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(household)?.groups.iter() {
      println!("{}", group.name);
    }
  }
  Ok(())
}

fn get_players(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  for household in sonos.get_households()?.iter() {
    for player in sonos.get_groups(household)?.players.iter() {
      println!("{}", player.name);
    }
  }
  Ok(())
}

fn household_arg() -> Arg {
  Arg::new("HOUSEHOLD")
    .long("household")
    .num_args(1)
    .value_name("INDEX")
    .help("Optional 0-based household index")
}

fn play_modes_args() -> Vec<Arg> {
  vec![
    Arg::new("REPEAT").short('r').long("repeat"),
    Arg::new("REPEAT_ONE").short('o').long("repeat-one"),
    Arg::new("CROSSFADE")
      .short('c')
      .long("crossfade")
      .help("Do crossfade between tracks"),
    Arg::new("SHUFFLE")
      .short('s')
      .long("shuffle")
      .help("Shuffle the tracks"),
  ]
}

trait ArgMatchesExt {
  fn household(&self, sonos: &mut Sonos) -> Result<Household>;
  fn favorite(&self, sonos: &mut Sonos, household: &Household) -> Result<Favorite>;
  fn group<'a>(&self, groups: &'a [Group]) -> Result<&'a Group>;
  fn player<'a>(&self, players: &'a [Player]) -> Result<&'a Player>;
  fn playlist(&self, sonos: &mut Sonos, household: &Household) -> Result<Playlist>;
  fn play_modes(&self) -> Option<PlayModes>;
}

impl ArgMatchesExt for ArgMatches {
  fn household(&self, sonos: &mut Sonos) -> Result<Household> {
    let households = sonos.get_households()?;
    match households.len() {
      0 => Err("No households found".into()),
      1 => Ok(households.into_iter().next().unwrap()),
      _ => match self.get_one::<String>("HOUSEHOLD") {
        None => Err("Multiple households found".into()),
        Some(index) => {
          let index = index
            .parse::<usize>()
            .chain_err(|| "Invalid household index")?;
          for (n, household) in households.into_iter().enumerate() {
            if n == index {
              return Ok(household);
            }
          }
          Err("Household out of range".into())
        }
      },
    }
  }
  fn favorite(&self, sonos: &mut Sonos, household: &Household) -> Result<Favorite> {
    let favorite_name = self.get_one::<String>("FAVORITE").unwrap();
    for favorite in sonos.get_favorites(household)?.items.into_iter() {
      if &favorite.name == favorite_name {
        return Ok(favorite);
      }
    }
    Err(ErrorKind::UnknownFavorite(favorite_name.to_string()).into())
  }
  fn playlist(&self, sonos: &mut Sonos, household: &Household) -> Result<Playlist> {
    let playlist_name = self.get_one::<String>("PLAYLIST").unwrap();
    for playlist in sonos.get_playlists(household)?.playlists.into_iter() {
      if &playlist.name == playlist_name {
        return Ok(playlist);
      }
    }
    Err(ErrorKind::UnknownPlaylist(playlist_name.to_string()).into())
  }
  fn group<'a>(&self, groups: &'a [Group]) -> Result<&'a Group> {
    let group_name = self.get_one::<String>("GROUP").unwrap();
    for group in groups.iter() {
      if &group.name == group_name {
        return Ok(group);
      }
    }
    Err(ErrorKind::UnknownGroup(group_name.to_string()).into())
  }
  fn player<'a>(&self, players: &'a [Player]) -> Result<&'a Player> {
    let player_name = self.get_one::<String>("PLAYER").unwrap();
    for player in players.iter() {
      if &player.name == player_name {
        return Ok(player);
      }
    }
    Err(ErrorKind::UnknownPlayer(player_name.to_string()).into())
  }
  fn play_modes(&self) -> Option<PlayModes> {
    let repeat = self.contains_id("REPEAT");
    let repeat_one = self.contains_id("REPEAT_ONE");
    let crossfade = self.contains_id("CROSSFADE");
    let shuffle = self.contains_id("SHUFFLE");
    if repeat || repeat_one || crossfade || shuffle {
      Some(PlayModes {
        repeat,
        repeat_one,
        crossfade,
        shuffle,
      })
    } else {
      None
    }
  }
}
