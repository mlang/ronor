use oauth2::basic::BasicClient;
use oauth2::{AccessToken, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use reqwest::{Client};
use std::collections::HashMap;
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
  }
  foreign_links {
    UrlParse(url::ParseError);
    SerdeJson(serde_json::error::Error);
    Request(reqwest::Error);
  }
}

pub fn oauth2(
  client_id: ClientId, client_secret: ClientSecret, redirect_url: RedirectUrl
) -> Result<BasicClient> {
  Ok(BasicClient::new(client_id, Some(client_secret),
      AuthUrl::new(Url::parse("https://api.sonos.com/login/v3/oauth")?),
      Some(TokenUrl::new(Url::parse("https://api.sonos.com/login/v3/oauth/access")?))
    ).set_redirect_url(redirect_url)
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
struct Households {
  households: Vec<Household>
}

#[derive(Deserialize)]
pub struct Household {
  id: HouseholdId
}

impl Household {
  pub fn get_groups(
    self: &Self, tok: &AccessToken
  ) -> Result<Groups> {
    let client = Client::new();
    Ok(
      client
      .get(&format!("https://api.ws.sonos.com/control/api/v1/households/{}/groups",
                    self.id))
      .bearer_auth(tok.secret()).send()?.error_for_status()?.json()?
    )
  }
}

pub fn get_households(
  tok: &AccessToken
) -> Result<Vec<Household>> {
  let client = Client::new();
  let households: Households = client
    .get("https://api.ws.sonos.com/control/api/v1/households")
    .bearer_auth(tok.secret()).send()?.error_for_status()?.json()?;
  Ok(households.households)
}

#[derive(Debug, Deserialize)]
pub struct Groups {
  pub groups: Vec<Group>,
  pub players: Vec<Player>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  pub coordinator_id: PlayerId,
  pub id: GroupId,
  pub playback_state: PlaybackState,
  pub player_ids: Vec<PlayerId>,
  pub name: String
}

impl Group {
  pub fn get_playback_status(
    &self, tok: &AccessToken
  ) -> Result<PlaybackStatus> {
    let client = Client::new();
    Ok(
      client
      .get(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playback",
                    self.id))
      .bearer_auth(tok.secret()).send()?.error_for_status()?.json()?
    )
  }
  pub fn get_volume(
    self: &Self, tok: &AccessToken
  ) -> Result<GroupVolume> {
    let client = Client::new();
    Ok(
      client
      .get(&format!("https://api.ws.sonos.com/control/api/v1/groups/{}/groupVolume",
                    self.id))
      .bearer_auth(tok.secret()).send()?.error_for_status()?.json()?
    )
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
  pub api_version: String,
  pub device_ids: Vec<String>,
  pub id: PlayerId,
  pub min_api_version: String,
  pub name: String,
  pub software_version: String,
  pub capabilities: Vec<Capability>
}

impl Player {
  pub fn get_volume(
    self: &Self, tok: &AccessToken
  ) -> Result<PlayerVolume> {
    let client = Client::new();
    Ok(
      client
      .get(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/playerVolume",
                    self.id))
      .bearer_auth(tok.secret()).send()?.error_for_status()?.json()?
    )
  }
  pub fn load_audio_clip(
    self: &Self, tok: &AccessToken,
    app_id: String, name: String,
    clip_type: Option<AudioClipType>, priority: Option<Priority>, volume: Option<u8>,
    http_authorization: Option<String>, stream_url: Option<String>
  ) -> Result<AudioClip> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("appId", app_id);
    params.insert("name", name);
    if let Some(clip_type) = clip_type {
      params.insert("clipType", serde_json::to_string(&clip_type)?);
    }
    if let Some(priority) = priority {
      params.insert("priority", serde_json::to_string(&priority)?);
    }
    if let Some(volume) = volume {
      params.insert("volume", volume.to_string());
    }
    if let Some(stream_url) = stream_url {
      params.insert("streamUrl", stream_url);
    }
    if let Some(http_authorization) = http_authorization {
      params.insert("httpAuthorization", http_authorization);
    }
    let mut audio_clip: AudioClip = client
      .post(&format!("https://api.ws.sonos.com/control/api/v1/players/{}/audioClip",
                     self.id))
      .bearer_auth(tok.secret())
      .json(&params)
      .send()?.error_for_status()?
      .json()?;
    audio_clip.player_id = Some(self.id.clone());
    Ok(audio_clip)
  }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct IntegrationConfig {
  pub client_id: ClientId,
  pub client_secret: ClientSecret,
  pub redirect_url: RedirectUrl
}

