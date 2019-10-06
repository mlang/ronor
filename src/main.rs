#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{Arg, ArgMatches, App, AppSettings, SubCommand};
use oauth2::{AuthorizationCode, ClientId, ClientSecret, RedirectUrl};
use ronor::{Sonos, Favorite, Group, Player, Playlist};
use rustyline::Editor;
use std::process::{Command, exit};
use std::convert::TryFrom;
use url::Url;
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
  }
}

fn build_cli() -> App<'static, 'static> {
  App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommand(SubCommand::with_name("init")
      .about("Initialise sonos integration configuration")
    ).subcommand(SubCommand::with_name("login")
      .about("Login with your sonos user account and authorize ronor")
    ).subcommand(SubCommand::with_name("completions").setting(AppSettings::Hidden)
      .about("Generates completion scripts for your shell")
      .arg(Arg::with_name("SHELL")
             .required(true)
    	     .possible_values(&["bash", "fish", "zsh"])
             .help("The shell to generate the script for"))
    ).subcommands(vec![
        get_favorites::build(), get_group_volume::build(),
	get_player_volume::build(), get_playlist::build(), get_playlists::build(),
	load_audio_clip::build(), load_favorite::build(),
	load_home_theater_playback::build(), load_line_in::build(),
	load_playlist::build(), now_playing::build(), pause::build(),
	play::build(), set_group_volume::build(), set_player_volume::build(),
	skip_to_next_track::build(), skip_to_previous_track::build(),
	speak::build(), toggle_play_pause::build()
      ]
    ).subcommand(SubCommand::with_name("get-groups")
      .about("Get list of groups")
    ).subcommand(SubCommand::with_name("get-players")
      .about("Get list of players")
    ).subcommand(SubCommand::with_name("get-playback-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("get-metadata-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    )
}

fn main() -> Result<()> {
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  //let players = player_names(&mut sonos)?;
  //let players: Vec<&str> = players.iter().map(|x| x.as_str()).collect();
  match build_cli().get_matches().subcommand() {
    ("init", Some(matches)) =>
      init(&mut sonos, matches),
    ("login", Some(matches)) =>
      login(&mut sonos, matches),
    ("completions", Some(matches)) => {
      let shell = matches.value_of("SHELL").unwrap();
      build_cli().gen_completions_to(
        "ronor",
        shell.parse::<>().unwrap(),
        &mut std::io::stdout()
      );
      Ok(())
    },
    ("get-favorites", Some(matches)) =>   get_favorites::run(&mut sonos, matches),
    ("get-group-volume", Some(matches)) =>
      get_group_volume::run(&mut sonos, matches),
    ("get-playback-status", Some(matches)) =>
      get_playback_status(&mut sonos, matches),
    ("get-player-volume", Some(matches)) =>
      get_player_volume::run(&mut sonos, matches),
    ("get-groups", Some(matches)) =>      get_groups(&mut sonos, matches),
    ("get-metadata-status", Some(matches)) =>
      get_metadata_status(&mut sonos, matches),
    ("get-players", Some(matches)) =>     get_players(&mut sonos, matches),
    ("get-playlist", Some(matches)) =>    get_playlist::run(&mut sonos, matches),
    ("get-playlists", Some(matches)) =>   get_playlists::run(&mut sonos, matches),
    ("load-audio-clip", Some(matches)) => load_audio_clip::run(&mut sonos, matches),
    ("load-favorite", Some(matches))   => load_favorite::run(&mut sonos, matches),
    ("load-home-theater-playback", Some(matches)) =>
      load_home_theater_playback::run(&mut sonos, matches),
    ("load-line-in", Some(matches)) =>    load_line_in::run(&mut sonos, matches),
    ("load-playlist", Some(matches)) =>   load_playlist::run(&mut sonos, matches),
    ("now-playing", Some(matches)) =>     now_playing::run(&mut sonos, matches),
    ("pause", Some(matches)) =>           pause::run(&mut sonos, matches),
    ("play", Some(matches)) =>            play::run(&mut sonos, matches),
    ("set-group-volume", Some(matches)) =>
      set_group_volume::run(&mut sonos, matches),
    ("set-player-volume", Some(matches)) =>
      set_player_volume::run(&mut sonos, matches),
    ("skip-to-previous-track", Some(matches)) =>
      skip_to_previous_track::run(&mut sonos, matches),
    ("skip-to-next-track", Some(matches)) =>
      skip_to_next_track::run(&mut sonos, matches),
    ("speak", Some(matches)) =>           speak::run(&mut sonos, matches),
    ("toggle-play-pause", Some(matches)) =>
      toggle_play_pause::run(&mut sonos, matches),
    _ => unreachable!()
  }
}

fn init(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  println!("Go to https://integration.sonos.com/ and create an account.");
  println!("");
  println!("Create a new control integration.");
  println!("");
  let mut console = Editor::<()>::new();
  let client_id = ClientId::new(console.readline("Client identifier: ")?);
  let client_secret = ClientSecret::new(console.readline("Client secret: ")?);
  let redirect_url = RedirectUrl::new(
    Url::parse(&console.readline("Redirection URL: ")?)?
  );
  sonos.set_integration_config(client_id, client_secret, redirect_url)?;
  println!("");
  println!("OK, we're ready to go.");
  println!("Now run 'ronor login' to authorize access to your Sonos user account.");
  Ok(())
}

fn login(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  let (auth_url, csrf_token) = sonos.authorization_url()?;
  let _lynx = Command::new("lynx")
    .arg("-nopause")
    .arg(auth_url.as_str())
    .status().expect("Failed to fire up browser.");
  println!("Token: {}", csrf_token.secret());
  let mut console = Editor::<()>::new();
  let code = console.readline("Code: ")?;
  sonos.authorize(AuthorizationCode::new(code.trim().to_string()))?;
  Ok(())
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
mod set_group_volume;
mod set_player_volume;
mod skip_to_next_track;
mod skip_to_previous_track;
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

fn player_names(sonos: &mut Sonos) -> Result<Vec<String>> {
  let mut players = Vec::new();
  for household in sonos.get_households()?.into_iter() {
    players.extend(
      sonos.get_groups(&household)?.players.into_iter().map(|p| p.name)
    );
  }
  Ok(players)
}


