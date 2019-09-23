use oauth2::basic::BasicClient;
use oauth2::{AccessToken, AuthorizationCode, AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};
use oauth2::reqwest::http_client;
use reqwest::{Client};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{read_to_string, write};
use serde::{Deserialize, Serialize};
use url::Url;

#[macro_use]
extern crate error_chain;

error_chain!{
  errors {
    UnknownPlayerId {
      description("PlayerId is unknown")
      display("Failed to find PlayerId")
    }
    IntegrationRequired {
      description("integration required for this operation")
      display("No integration configuration found")
    }
    TokenRequired {
      description("access_token required for this operation")
      display("No access_token available")
    }
    TokenExpired {
      description("access_token is likely expired")
      display("Failed to call with status code 401")
    }
  }
  foreign_links {
    IO(std::io::Error);
    TOMLDeserialization(toml::de::Error);
    TOMLSerialization(toml::ser::Error);
    UrlParse(url::ParseError);
    SerdeJson(serde_json::error::Error);
    Request(reqwest::Error);
  }
}

fn oauth2(
  client_id: &ClientId, client_secret: &ClientSecret, redirect_url: &RedirectUrl
) -> Result<BasicClient> {
  Ok(BasicClient::new(client_id.clone(), Some(client_secret.clone()),
      AuthUrl::new(Url::parse("https://api.sonos.com/login/v3/oauth")?),
      Some(TokenUrl::new(Url::parse("https://api.sonos.com/login/v3/oauth/access")?))
    ).set_redirect_url(redirect_url.clone())
  )
}

#[derive(Debug, Deserialize)]
pub struct HouseholdId(String);

impl std::fmt::Display for HouseholdId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Deserialize)]
pub struct GroupId(String);

impl std::fmt::Display for GroupId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerId(String);

impl std::fmt::Display for PlayerId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FavoriteId(String);

impl std::fmt::Display for FavoriteId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioClipType {
  Chime, Custom
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
  Playback,
  Cloud,
  HtPlayback,
  HtPowerState,
  Airplay,
  LineIn,
  AudioClip,
  Voice,
  SpeakerDetection,
  FixedVolume
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaybackState {
  PlaybackStateIdle,
  PlaybackStatePaused,
  PlaybackStateBuffering,
  PlaybackStatePlaying
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
  Low, High
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Households {
  households: Vec<Household>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Household {
  id: HouseholdId
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Groups {
  pub groups: Vec<Group>,
  pub players: Vec<Player>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  pub coordinator_id: PlayerId,
  pub id: GroupId,
  pub playback_state: PlaybackState,
  pub player_ids: Vec<PlayerId>,
  pub area_ids: Vec<String>,
  pub name: String
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Player {
  pub api_version: String,
  pub device_ids: Vec<String>,
  pub id: PlayerId,
  pub min_api_version: String,
  pub name: String,
  pub software_version: String,
  pub capabilities: Vec<Capability>,
  pub websocket_url: String
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AudioClip {
  app_id: String,
  name: String,
  clip_type: Option<AudioClipType>,
  error_code: Option<String>,
  id: String,
  priority: Option<Priority>,
  status: Option<String>,
  #[serde(skip)]
  player_id: Option<PlayerId>
}

impl AudioClip {
  pub fn cancel(
    self: &Self, tok: &AccessToken
  ) -> Result<()> {
    if let Some(player_id) = &self.player_id {
      let client = Client::new();
      client
        .delete(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/audioClip/{}",
                          player_id, self.id))
        .bearer_auth(tok.secret()).send()?.error_for_status()?;
      Ok(())
    } else {
      Err(ErrorKind::UnknownPlayerId.into())
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct GroupVolume {
  volume: u8,
  muted: bool,
  fixed: bool
}

#[derive(Debug, Deserialize)]
pub struct PlayerVolume {
  volume: u8,
  muted: bool,
  fixed: bool
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatus {
  playback_state: PlaybackState,
  queue_version: Option<String>,
  item_id: Option<String>,
  position_millis: i64,
  play_modes: PlayModes,
  available_playback_actions: AvailablePlaybackActions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayModes {
  repeat: bool,
  repeat_one: bool,
  crossfade: bool,
  shuffle: bool
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePlaybackActions {
  can_skip: bool,
  can_skip_back: bool,
  can_seek: bool,
  can_repeat: bool,
  can_repeat_one: bool,
  can_crossfade: bool,
  can_shuffle: bool
}

#[derive(Debug, Deserialize)]
pub struct Favorites {
  version: String,
  items: Vec<Favorite>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
  id: FavoriteId,
  name: String,
  description: Option<String>,
  image_url: Option<String>,
  service: Service
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
  name: String,
  id: Option<String>,
  image_url: Option<String>
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadFavorite {
  favorite_id: FavoriteId,
  play_on_completion: bool,
  play_modes: Option<PlayModes>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IntegrationConfig {
  pub client_id: ClientId,
  pub client_secret: ClientSecret,
  pub redirect_url: RedirectUrl
}

#[derive(Clone, Deserialize, Serialize)]
struct Tokens {
  access_token: AccessToken,
  refresh_token: RefreshToken
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicObjectId {
  service_id: Option<String>,
  object_id: String,
  account_id: Option<String>
}
  
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
  name: Option<String>,
  #[serde(rename = "type")]
  type_: Option<String>,
  id: Option<MusicObjectId>,
  service: Option<Service>,
  image_url: Option<String>,
  tags: Option<Vec<String>>
}
  
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
  id: Option<String>,
  track: Track,
  deleted: Option<bool>,
  policies: Policies
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Policies {
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
  can_crossfade: Option<bool>,
  can_skip: Option<bool>,
  duration_millis: Option<i32>,
  id: Option<MusicObjectId>,
  image_url: Option<String>,
  name: Option<String>,
  replay_gain: Option<f32>,
  tags: Vec<String>,
  service: Service
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatus {
  container: Option<Container>,
  current_item: Option<Item>,
  next_item: Option<Item>,
  stream_info: Option<String>,
}

pub struct Sonos {
  integration: Option<IntegrationConfig>,
  integration_path: Option<std::path::PathBuf>,
  tokens: Option<Tokens>,
  tokens_path: Option<std::path::PathBuf>
}

impl Sonos {
  pub fn is_registered(self: &Self) -> bool { self.integration.is_some() }

  pub fn is_authorized(self: &Self) -> bool { self.tokens.is_some() }

  pub fn authorization_url(self: &Self
  ) -> Result<(url::Url, oauth2::CsrfToken)> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        Ok(auth.authorize_url(oauth2::CsrfToken::new_random).add_scope(oauth2::Scope::new("playback-control-all".to_string())).url())
      },
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }

  pub fn authorize(self: &mut Self,
    code: AuthorizationCode
  ) -> Result<()> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        let token_result = auth.exchange_code(code).request(http_client).unwrap();
        if let Some(refresh_token) = token_result.refresh_token() {
          self.tokens = Some(Tokens { access_token: token_result.access_token().clone()
                              , refresh_token: refresh_token.clone()
                               });
          let toml = toml::to_string_pretty(&self.tokens.as_ref().unwrap())?;
          if let Some(path) = &self.tokens_path {
            write(path, toml)?;
          }
          Ok(())
        } else {
          Err("No refresh token received".into())
        }
      },
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }
  
  pub fn refresh_token(self: &mut Self) -> Result<&mut Self> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        match &self.tokens {
          Some(tokens) => {
            let token_response = auth.exchange_refresh_token(&tokens.refresh_token)
              .request(http_client).unwrap();
            self.tokens = Some(Tokens {
                access_token: token_response.access_token().clone(),
                refresh_token: token_response.refresh_token().unwrap_or(&tokens.refresh_token).clone()
              }
            );
            if let Some(tokens_path) = &self.tokens_path {
              write(tokens_path, toml::to_string_pretty(self.tokens.as_ref().unwrap())?)?;
            }
            Ok(self)
          },
          None => Err(ErrorKind::TokenRequired.into())
        }
      },
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }

  fn maybe_refresh<T>(
    self: &mut Self,
    call: &dyn Fn(&AccessToken) -> Result<reqwest::Response>,
    convert: &dyn Fn(reqwest::Response) -> Result<T>
  ) -> Result<T> {
    match &self.tokens {
      Some(tokens) => convert({
        let response = call(&tokens.access_token)?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
          self.refresh_token()?;
          call(&self.tokens.as_ref().unwrap().access_token)?
        } else {
          response
        }
      }.error_for_status()?),
      None => Err(ErrorKind::TokenRequired.into())
    }
  }

  pub fn get_households(self: &mut Self
  ) -> Result<Vec<Household>> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get("https://api.ws.sonos.com/control/api/v1/households")
          .bearer_auth(access_token.secret())
          .send()?
      )
    }, &|mut response| {
      let households: Households = response.json()?;
      Ok(households.households)
    })
  }
  pub fn get_groups(self: &mut Self,
    household: &Household
  ) -> Result<Groups> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/households/{}/groups", household.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }
  pub fn get_favorites(self: &mut Self,
    household: &Household
  ) -> Result<Favorites> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/households/{}/favorites",
                        household.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }
  pub fn get_playback_status(self: &mut Self,
    group: &Group
  ) -> Result<PlaybackStatus> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback",
                        group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }
  pub fn get_metadata_status(self: &mut Self,
    group: &Group
  ) -> Result<MetadataStatus> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playbackMetadata",
                        group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| {
      Ok(response.json()?)
    }
    )
  }
  pub fn load_favorite(self: &mut Self,
    group: &Group,
    favorite: &Favorite,
    play_on_completion: bool,
    play_modes: &Option<PlayModes>
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let params = LoadFavorite {
        favorite_id: favorite.id.clone(),
        play_on_completion: play_on_completion,
        play_modes: play_modes.clone()
      };
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/favorites", group.id))
          .bearer_auth(access_token.secret()).json(&params).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn get_group_volume(self: &mut Self,
    group: &Group
  ) -> Result<GroupVolume> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/groupVolume", group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }
  pub fn set_group_volume(self: &mut Self,
    group: &Group,
    volume: u8
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      let mut params = HashMap::new();
      params.insert("volume", volume);
      Ok(
        client
          .put(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/groupVolume", group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn play(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback/play",
	                 group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback/pause",
	                 group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn toggle_play_pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback/togglePlayPause",
	                 group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn skip_to_next_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback/skipToNextTrack",
	                 group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn skip_to_previous_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback/skipToPreviousTrack",
	                 group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }

  pub fn get_player_volume(self: &mut Self,
    player: &Player
  ) -> Result<PlayerVolume> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/playerVolume",
                        player.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }
  pub fn set_player_volume(self: &mut Self,
    player: &Player,
    volume: u8
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      let mut params = HashMap::new();
      params.insert("volume", volume);
      Ok(
        client
          .put(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/playerVolume", player.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn load_audio_clip(self: &mut Self,
    player: &Player, app_id: String, name: String,
    clip_type: Option<AudioClipType>, priority: Option<Priority>, volume: Option<u8>,
    http_authorization: Option<String>, stream_url: Option<String>
  ) -> Result<AudioClip> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      let mut params = HashMap::new();
      params.insert("appId", app_id.clone());
      params.insert("name", name.clone());
      if let Some(clip_type) = &clip_type {
        params.insert("clipType", serde_json::to_string(&clip_type)?);
      }
      if let Some(priority) = &priority {
        params.insert("priority", serde_json::to_string(&priority)?);
      }
      if let Some(volume) = volume {
        params.insert("volume", volume.to_string());
      }
      if let Some(stream_url) = &stream_url {
        params.insert("streamUrl", stream_url.clone());
      }
      if let Some(http_authorization) = &http_authorization {
        params.insert("httpAuthorization", http_authorization.clone());
      }
      Ok(
        client
          .post(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/audioClip",
                         player.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|mut response| {
      let mut audio_clip: AudioClip = response.json()?;
      audio_clip.player_id = Some(player.id.clone());
      Ok(audio_clip)
    })
  }
}

impl TryFrom<xdg::BaseDirectories> for Sonos {
  type Error = Error;
  fn try_from(xdg_dirs: xdg::BaseDirectories) -> Result<Self> {
    let integration_config_path = xdg_dirs.place_config_file("sonos_integration.toml")?;
    let tokens_config_path = xdg_dirs.place_config_file("sonos_tokens.toml")?;
    match read_to_string(&integration_config_path).and_then(|s| Ok(toml::from_str(&s)?)) {
      Ok(integration) => match read_to_string(&tokens_config_path).and_then(|s| Ok(toml::from_str(&s)?)) {
        Ok(tokens) => Ok(Sonos {
          integration: Some(integration),
          integration_path: Some(integration_config_path),
          tokens: Some(tokens),
          tokens_path: Some(tokens_config_path)
        }),
        Err(_) => Ok(Sonos {
          integration: Some(integration),
          integration_path: Some(integration_config_path),
          tokens: None,
          tokens_path: Some(tokens_config_path)
        })
      },
      Err(_) => match read_to_string(&tokens_config_path).and_then(|s| Ok(toml::from_str(&s)?)) {
        Ok(tokens) => Ok(Sonos {
          integration: None,
          integration_path: Some(integration_config_path),
          tokens: Some(tokens),
          tokens_path: Some(tokens_config_path)
        }),
        Err(_) => Ok(Sonos {
          integration: None,
          integration_path: Some(integration_config_path),
          tokens: None,
          tokens_path: Some(tokens_config_path)
        })
      }
    }
  }
}