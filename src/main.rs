#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{Arg, ArgMatches, App, AppSettings};
use ronor::{Sonos, Favorite, Group, Household, Player, Playlist, PlayModes};
use std::process::{exit};
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

fn build_cli() -> App<'static, 'static> {
  App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommands(vec![init::build(), login::build()])
    .subcommands(vec![
        get_favorites::build(), get_playlist::build(), get_playlists::build(),
        get_volume::build(), inventory::build(), load_audio_clip::build(), load_favorite::build(),
        load_home_theater_playback::build(), load_line_in::build(),
        load_playlist::build(), modify_group::build(), now_playing::build(), pause::build(),
        play::build(), seek::build(), set_mute::build(), set_volume::build(),
        skip::build(), speak::build(), toggle_play_pause::build()
      ])
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
  macro_rules! match_subcommands {
    ($e:expr $(, $mod:ident)+) => {
      match $e {
        $(($mod::NAME, Some(matches)) => $mod::run(&mut sonos, matches),)+
        _ => unimplemented!()
      }
    }
  }
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
    (cmd, matches) => match_subcommands!((cmd, matches),
      init, login,
      get_favorites, get_playlist, get_playlists, get_volume, inventory,
      load_audio_clip, load_favorite, load_home_theater_playback, load_line_in,
      load_playlist, modify_group, now_playing, pause, play, seek, set_mute,
      set_volume, skip, speak, toggle_play_pause
    )
  }
}

macro_rules! with_authorization {
  ($sonos:ident, $code:block) => {
    if !$sonos.is_authorized() {
      return Err("Not authorized".into());
    } else $code
  };
}

mod init;
mod login;
mod get_favorites;
mod get_playlist;
mod get_playlists;
mod get_volume;
mod inventory;
mod load_audio_clip;
mod load_favorite;
mod load_home_theater_playback;
mod load_line_in;
mod load_playlist;
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

fn get_playback_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
        matches.value_of("GROUP").map_or(true, |name| name == group.name)
      ) {
        found = true;
        println!("{:?} => {:#?}", group.name, sonos.get_playback_status(&group)?);
      }
    }
    if matches.value_of("GROUP").is_some() && !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn get_metadata_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
        matches.value_of("GROUP").map_or(true, |name| name == group.name)
      ) {
        found = true;
        println!("{:?} => {:#?}", group.name, sonos.get_metadata_status(&group)?);
      }
    }
    if matches.value_of("GROUP").is_some() && !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn get_groups(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        println!("{}", group.name);
      }
    }
    Ok(())
  })
}

fn get_players(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for player in sonos.get_groups(&household)?.players.iter() {
        println!("{}", player.name);
      }
    }
    Ok(())
  })
}

fn household_arg() -> Arg<'static, 'static> {
  Arg::with_name("HOUSEHOLD")
    .long("household").takes_value(true).value_name("INDEX")
    .help("Optional 0-based household index")
}

trait ArgMatchesExt {
  fn household(self: &Self, sonos: &mut Sonos) -> Result<Household>;
  fn favorite(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Favorite>;
  fn group<'a>(self: &Self, groups: &'a [Group]) -> Result<&'a Group>;
  fn player<'a>(self: &Self, players: &'a [Player]) -> Result<&'a Player>;
  fn playlist(self: &Self, sonos: &mut Sonos, household: &Household) -> Result<Playlist>;
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
          let mut n = 0;
          for household in households.into_iter() {
            if n == index {
              return Ok(household)
            }
            n += 1;
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
}

fn play_modes(matches: &ArgMatches) -> Option<PlayModes> {
  let repeat = matches.is_present("REPEAT");
  let repeat_one = matches.is_present("REPEAT_ONE");
  let crossfade = matches.is_present("CROSSFADE");
  let shuffle = matches.is_present("SHUFFLE");
  if repeat || repeat_one || crossfade || shuffle {
    Some(PlayModes { repeat, repeat_one, crossfade, shuffle })
  } else {
    None
  }
}
