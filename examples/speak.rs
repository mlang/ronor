use oauth2::AccessToken;
use std::fs::{File};
use std::io::{Read};
use std::process::{Command, Stdio};
use ronor::*;

fn main() -> Result<()> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix("ronor").unwrap();
  let config_path = xdg_dirs.place_config_file("access_token").unwrap();
  let access_token = match File::open(&config_path) {
    Ok(mut file) => {
      let mut tok = String::new();
      file.read_to_string(&mut tok).expect("Error reading token from file");
      AccessToken::new(tok)
    },
    Err(_) => panic!("No access token found")
  };

  speak_stdin(&access_token, "Wohnzimmer", "de", 250, 75)
}

fn speak_stdin(
  tok: &AccessToken, player_name: &str, voice: &str, wpm: u16, volume: u8
) -> Result<()> {
  let espeak = Command::new("espeak")
    .args(&[ "-a", &(volume * 2).to_string()
           , "-s", &wpm.to_string()
           , "-v", voice
           , "-w", "/dev/stdout", "--stdin"])
    .stdout(Stdio::piped()).spawn()
    .expect("Failed to spawn espeak");
  if let Some(stdout) = espeak.stdout {
    let ffmpeg = Command::new("ffmpeg")
      .args(&["-i", "-", "-v", "fatal", "-b:a", "96k", "-f", "mp3", "-"])
      .stdin(stdout).stdout(Stdio::piped()).spawn()
      .expect("Failed to spawn ffmpeg");
    if let Some(stdout) = ffmpeg.stdout {
      let curl = Command::new("curl")
        .args(&["--upload-file", "-", "https://transfer.sh/espeak.mp3"])
        .stdin(stdout).output()
        .expect("Failed curl output");
      if curl.status.success() {
        let url = String::from_utf8_lossy(&curl.stdout);

        let households = get_households(tok)?;
        for household in households {
          let groups = household.get_groups(tok)?;
          for player in groups.players.iter().filter(|p| p.name == player_name) {
            let _clip = player.load_audio_clip(tok,
              String::from("guru.blind"), String::from("ping"), None, None, None,
              None, Some(url.to_string())
            )?;
          }
        }
      }
    }
  }
  Ok(())
}
