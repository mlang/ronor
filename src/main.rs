#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{Arg, ArgMatches, App, AppSettings};
use ronor::{Sonos, Favorite, Group, Player, Playlist};
use std::process::{exit};
use std::convert::TryFrom;
use xdg::BaseDirectories;

error_chain! {
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
    //HumanTime(humantime::Error);
  }
}

fn build_cli() -> App<'static, 'static> {
  App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommands(vec![init::build(), login::build()])
    .subcommands(vec![
        get_favorites::build(), get_group_volume::build(),
        get_player_volume::build(), get_playlist::build(),
        get_playlists::build(), load_audio_clip::build(),
        load_favorite::build(), load_home_theater_playback::build(),
        load_line_in::build(), load_playlist::build(), now_playing::build(),
        pause::build(), play::build(), seek::build(), set_mute::build(),
        set_volume::build(), skip::build(), speak::build(), toggle_play_pause::build()
      ])
    .subcommand(App::new("get-groups")
      .about("Get list of groups"))
    .subcommand(App::new("get-players")
      .about("Get list of players"))
    .subcommand(App::new("get-playback-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP")))
    .subcommand(App::new("get-metadata-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP")))
    .subcommand(App::new("completions").setting(AppSettings::Hidden)
      .about("Generates completion scripts for your shell")
      .arg(Arg::with_name("SHELL")
             .required(true)
             .possible_values(&["bash", "fish", "zsh"])
             .help("The shell to generate the script for")))
}

fn main() -> Result<()> {
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
      init, login, get_favorites, get_group_volume, get_player_volume,
      get_playlist, get_playlists, load_audio_clip, load_favorite,
      load_home_theater_playback, load_line_in, load_playlist, now_playing,
      pause, play, seek, set_mute, set_volume, skip, speak, toggle_play_pause
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

macro_rules! with_favorite {
  ($sonos:ident, $matches:ident, $favorite:ident, $code:block) => {
    if let Some($favorite) = find_favorite_by_name(
      $sonos, $matches.value_of("FAVORITE").unwrap()
    )? $code
    else {
      return Err("Favorite not found".into());
    }
  }
}

macro_rules! with_group {
  ($sonos:ident, $matches:ident, $group:ident, $code:block) => {
    if let Some($group) = find_group_by_name(
      $sonos, $matches.value_of("GROUP").unwrap()
    )? $code
    else {
      return Err("Group not found".into());
    }
  }
}

macro_rules! with_player {
  ($sonos:ident, $matches:ident, $player:ident, $code:block) => {
    if let Some($player) = find_player_by_name(
      $sonos, $matches.value_of("PLAYER").unwrap()
    )? $code
    else {
      return Err("Player not found".into());
    }
  }
}

macro_rules! with_playlist {
  ($sonos:ident, $matches:ident, $playlist:ident, $code:block) => {
    if let Some($playlist) = find_playlist_by_name(
      $sonos, $matches.value_of("PLAYLIST").unwrap()
    )? $code
    else {
      return Err("Playlist not found".into());
    }
  }
}

mod init;
mod login;
mod get_favorites;
mod get_group_volume;
mod get_player_volume;
mod get_playlist;
mod get_playlists;
mod load_audio_clip;
mod load_favorite;
mod load_home_theater_playback;
mod load_line_in;
mod load_playlist;
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

fn find_group_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Group>> {
  for household in sonos.get_households()?.into_iter() {
    for group in sonos.get_groups(&household)?.groups.into_iter() {
      if group.name == name {
        return Ok(Some(group))
      }
    }
  }
  Ok(None)
}

fn find_player_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Player>> {
  for household in sonos.get_households()?.into_iter() {
    for player in sonos.get_groups(&household)?.players.into_iter() {
      if player.name == name {
        return Ok(Some(player))
      }
    }
  }
  Ok(None)
}

fn find_favorite_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Favorite>> {
  for household in sonos.get_households()?.into_iter() {
    for favorite in sonos.get_favorites(&household)?.items.into_iter() {
      if favorite.name == name {
        return Ok(Some(favorite))
      }
    }
  }
  Ok(None)
}

fn find_playlist_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Playlist>> {
  for household in sonos.get_households()?.into_iter() {
    for playlist in sonos.get_playlists(&household)?.playlists.into_iter() {
      if playlist.name == name {
        return Ok(Some(playlist))
      }
    }
  }
  Ok(None)
}
