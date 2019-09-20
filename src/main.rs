use oauth2::{
  AccessToken,
  AuthorizationCode,
  CsrfToken,
  RefreshToken,
  Scope,
  TokenResponse,
};
use oauth2::reqwest::http_client;
use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{stdin, Read};
use std::process::Command;
use ronor::*;

#[derive(Deserialize, Serialize)]
struct Tokens {
  access_token: AccessToken,
  refresh_token: RefreshToken
}

fn main() -> Result<()> {
  let xdg_dirs = xdg::BaseDirectories::with_prefix("ronor").unwrap();
  let integration_config_path = xdg_dirs.place_config_file("integration.toml").expect("Cannot create configuration directory");
  let tokens_config_path = xdg_dirs.place_config_file("tokens.toml").expect("cannot create configuration directory");

  let access_token = match File::open(&tokens_config_path) {
    Ok(mut file) => {
      let mut tok = String::new();
      file.read_to_string(&mut tok).expect("Error reading token from file");
      let tokens: Tokens = toml::from_str(&tok).expect("Failed to parse toml");
      tokens.access_token.clone()
    },
    Err(_) => {
      let str = std::fs::read_to_string(integration_config_path)
        .expect("Failed to read integration configuration");
      let integration_config: IntegrationConfig = toml::from_str(&str).expect("Failed to parse integration configuration");
      let auth = oauth2(
        integration_config.client_id,
        integration_config.client_secret,
        integration_config.redirect_url
      )?;
      let (auth_url, csrf_token) = auth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("playback-control-all".to_string()))
        .url();
      let _lynx = Command::new("lynx")
        .arg("-nopause")
        .arg(auth_url.as_str())
        .status().expect("Failed to fire up browser.");
      println!("Token: {}", csrf_token.secret());
      let mut code = String::new();
      stdin().read_line(&mut code).expect("unable to read user input");
      let code = AuthorizationCode::new(code.to_string().trim().to_string());
      let token_result = auth.exchange_code(code).request(http_client)
        .expect("Token result error");
      if let Some(refresh_token) = token_result.refresh_token() {
        let tokens = Tokens { access_token: token_result.access_token().clone()
                            , refresh_token: refresh_token.clone()
                            };
        let toml = toml::to_string_pretty(&tokens).unwrap();
        std::fs::write(&tokens_config_path, toml).expect("Failed to write tokens");
      }
      token_result.access_token().clone()
    }
  };

  let households = get_households(&access_token)?;
  if households.len() == 1 {
    println!("{:?}", households[0].get_favorites(&access_token)?);
    let groups = households[0].get_groups(&access_token)?;
    println!("{:?}", groups);
    for player in groups.players {
      let clip = player.load_audio_clip(&access_token,
        String::from("guru.blind"), String::from("ping"), None, None, None, None, None
      )?;
      println!("{:?}", clip);
      println!("{:?}", player.get_volume(&access_token));
    }
    for group in groups.groups {
      let group_volume = group.get_volume(&access_token)?;
      println!("{:?}", group_volume);
      println!("{:?}", group.get_playback_status(&access_token)?);
    }
  }

  Ok(())
}
