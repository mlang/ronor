use clap::{Arg, App, SubCommand};
use oauth2::AuthorizationCode;
use ronor::{IntegrationConfig, Sonos};
use rustyline::Editor;
use std::process::{Command, Stdio};
use std::convert::TryFrom;
use xdg::BaseDirectories;

#[macro_use]
extern crate error_chain;

error_chain! {
  links {
    API(ronor::Error, ronor::ErrorKind);
  }
  foreign_links {
    IO(std::io::Error);
    XDG(xdg::BaseDirectoriesError);
    ReadLine(rustyline::error::ReadlineError);
  }
}

fn main() -> Result<()> {
  let matches = App::new("ronor")
    .about("Does awesome things")
    .subcommand(SubCommand::with_name("login")
      .about("Refresh the access token")
    ).subcommand(SubCommand::with_name("refresh")
      .about("Refresh the access token")
    ).subcommand(SubCommand::with_name("speak")
      .about("Refresh the access token")
    ).subcommand(SubCommand::with_name("getHouseholds")
      .about("Get households (DEBUG)")
    )
    .get_matches();
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  if let Some(matches) = matches.subcommand_matches("login") {
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
  } else if let Some(matches) = matches.subcommand_matches("refresh") {
    if !sonos.is_registered() {
      println!("ronor is not registered, run authroize");
    } else if !sonos.is_authorized() {
      println!("Not authroized, can not refresh");
    } else {
      sonos.refresh_token()?;
    }
    Ok(())
  } else if let Some(matches) = matches.subcommand_matches("speak") {
    let espeak = Command::new("espeak")
    .args(&[ "-a", "150"
        , "-s", "250"
        , "-v", "de"
        , "-w", "/dev/stdout", "--stdin"])
    .stdout(Stdio::piped()).spawn()?;
    if let Some(stdout) = espeak.stdout {
      let ffmpeg = Command::new("ffmpeg")
        .args(&["-i", "-", "-v", "fatal", "-b:a", "96k", "-f", "mp3", "-"])
        .stdin(stdout).stdout(Stdio::piped()).spawn()?;
      if let Some(stdout) = ffmpeg.stdout {
        let curl = Command::new("curl")
          .args(&["--upload-file", "-", "https://transfer.sh/espeak.mp3"])
          .stdin(stdout).output()?;
        if curl.status.success() {
          let url = String::from_utf8_lossy(&curl.stdout);

          let households = sonos.get_households()?;
          for household in households {
            let groups = sonos.get_groups(&household)?;
            for player in groups.players.iter().filter(|p| p.name == "Wohnzimmer") {
              let _clip = sonos.load_audio_clip(&player,
                String::from("guru.blind"), String::from("ping"), None, None, None,
                None, Some(url.to_string())
              )?;
            }
          }
        }
      }
    }
    Ok(())
  } else if let Some(matches) = matches.subcommand_matches("getHouseholds") {
    if !sonos.is_authorized() {
      println!("Not authroized, can not refresh");
    } else {
      println!("{:?}", sonos.get_households());
    }
    Ok(())
  } else {
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        println!("{:?}", sonos.get_metadata_status(&group));
      }
    }
    Ok(())
  }
}
