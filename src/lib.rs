#![warn(rust_2018_idioms)]
use error_chain::error_chain;
use oauth2::basic::{BasicClient, BasicErrorResponse};
use oauth2::reqwest::http_client;
use oauth2::{
  AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
  RedirectUrl, RefreshToken, RequestTokenError, Scope, TokenResponse, TokenUrl
};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{read_to_string, write};
use std::str::FromStr;
use url::Url;

error_chain! {
  errors {
    MissingCapability(c: Capability) {
      description("missing capability")
      display("Player is missing the {:?} capability", c)
    }
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

const AUTH_URL: &str = "https://api.sonos.com/login/v3/oauth";
const TOKEN_URL: &str = "https://api.sonos.com/login/v3/oauth/access";
const PREFIX: &str = "https://api.ws.sonos.com/control/api/v1/";

macro_rules! control_v1 {
  ($($arg:tt)*) => {
    (PREFIX.to_string() + &format!($($arg)*)).as_str()
  }
}

fn oauth2(
  client_id: &ClientId,
  client_secret: &ClientSecret,
  redirect_url: &RedirectUrl
) -> Result<BasicClient> {
  Ok(
    BasicClient::new(
      client_id.clone(),
      Some(client_secret.clone()),
      AuthUrl::new(AUTH_URL.to_string())?,
      Some(TokenUrl::new(TOKEN_URL.to_string())?)
    )
    .set_redirect_url(redirect_url.clone())
  )
}

macro_rules! ids {
  ($name:ident) => {
    #[derive (Clone, Debug, Deserialize, PartialEq, Serialize)]
    pub struct $name(String);

    impl $name {
      pub fn new(s: String) -> Self { Self(s) }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }
  };
  ($name:ident, $($more:ident),+) => { ids!($name); ids!($($more),+); }
}

ids!(HouseholdId, GroupId, PlayerId, FavoriteId, PlaylistId, AudioClipId);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioClipType {
  Chime,
  Custom
}

impl FromStr for AudioClipType {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Chime" => Ok(AudioClipType::Chime),
      "Custom" => Ok(AudioClipType::Custom),
      _ => Err("no match".into())
    }
  }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
  /// The player can produce audio.
  Playback,
  /// The player can send commands and receive events over the internet.
  Cloud,
  /// The player is a home theater source.
  /// It can reproduce the audio from a home theater system,
  /// typically delivered by S/PDIF or HDMI.
  HtPlayback,
  /// The player can control the home theater power state.
  /// For example, it can switch a connected TV on or off.
  HtPowerState,
  /// The player can host AirPlay streams.
  /// This capability is present when the device is advertising AirPlay support.
  Airplay,
  /// The player has an analog line-in.
  LineIn,
  /// The device is capable of playing audio clip notifications.
  AudioClip,
  Voice,
  SpeakerDetection,
  FixedVolume
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum PlaybackState {
  /// Playback is not playing or paused, such as when the queue is empty
  /// or a source cannot be paused (such as streaming radio).
  #[serde(rename = "PLAYBACK_STATE_IDLE")]
  Idle,
  /// Playback is paused while playing content that can be paused and resumed.
  #[serde(rename = "PLAYBACK_STATE_PAUSED")]
  Paused,
  /// The group is buffering audio.
  /// This is a transitional state before the audio starts playing.
  #[serde(rename = "PLAYBACK_STATE_BUFFERING")]
  Buffering,
  /// The group is playing audio.
  #[serde(rename = "PLAYBACK_STATE_PLAYING")]
  Playing
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
  Low,
  High
}

impl FromStr for Priority {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Low" => Ok(Priority::Low),
      "High" => Ok(Priority::High),
      _ => Err("no match".into())
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Tag {
  #[serde(rename = "TAG_EXPLICIT")]
  Explicit
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TvPowerState {
  On,
  Standby
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Households {
  households: Vec<Household>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Household {
  pub id: HouseholdId,
  pub name: Option<String>
}

/// Describes the current set of logical players and groups in the household.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Groups {
  /// A list of groups in the household.
  pub groups: Vec<Group>,
  /// A list of the players in the household.
  pub players: Vec<Player>,
  pub partial: bool
}

/// Describes one group in a household.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  /// The ID of the player acting as the group coordinator for the group.
  pub coordinator_id: PlayerId,
  /// The ID of the group.
  pub id: GroupId,
  /// The playback state corresponding to the group.
  pub playback_state: PlaybackState,
  /// The IDs of the primary players in the group.
  /// For example, only one player from each set of players bonded as a stereo
  /// pair or as satellites to a home theater setup. Each element is the ID
  /// of a player. This list includes the coordinator_id.
  pub player_ids: Vec<PlayerId>,
  #[serde(default = "Vec::new")]
  pub area_ids: Vec<String>,
  /// The display name for the group, such as “Living Room” or “Kitchen + 2”.
  pub name: String
}

/// Describes a group after it has been modified.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedGroup {
  /// The ID of the player acting as the group coordinator for the group.
  pub coordinator_id: PlayerId,
  /// The ID of the group.
  pub id: GroupId,
  /// The IDs of the primary players in the group.
  /// For example, only one player from each set of players bonded as a stereo
  /// pair or as satellites to a home theater setup. Each element is the ID
  /// of a player. This list includes the coordinator_id.
  pub player_ids: Vec<PlayerId>,
  #[serde(default = "Vec::new")]
  pub area_ids: Vec<String>,
  /// The display name for the group, such as “Living Room” or “Kitchen + 2”.
  pub name: String
}

/// Describes one logical speaker in a household.
/// A logical speaker could be a single stand-alone device or a set of bonded
/// devices. For example, two players bonded as a stereo pair, two
/// surrounds and a SUB bonded with a PLAYBAR in a home theater setup,
/// or a player bonded with a SUB.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Player {
  pub is_unregistered: bool,
  /// The highest API version supported by the player.
  pub api_version: String,
  /// The IDs of all bonded devices corresponding to this logical player.
  pub device_ids: Vec<String>,
  /// The ID of the player.
  pub id: PlayerId,
  /// The lowest API version supported by the player.
  pub min_api_version: String,
  /// The display name for the player.
  /// For example, “Living Room”, “Kitchen”, or “Dining Room”.
  pub name: String,
  /// The version of the software running on the device.
  pub software_version: String,
  /// The set of capabilities provided by the player.
  pub capabilities: Vec<Capability>,
  /// The secure WebSocket URL for the device.
  pub websocket_url: String,
  /// This is present if airplay is currently in use.
  pub virtual_line_in_source: Option<VirtualLineInSource>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct VirtualLineInSource {
  #[serde(rename = "type")]
  pub type_: VirtualLineInSourceType
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VirtualLineInSourceType {
  Airplay
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AudioClip {
  pub app_id: String,
  pub name: String,
  pub clip_type: Option<AudioClipType>,
  pub id: AudioClipId,
  pub priority: Option<Priority>,
  pub status: Option<String>,
  #[serde(skip)]
  player_id: Option<PlayerId>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVolume {
  pub volume: u8,
  pub muted: bool,
  pub fixed: bool
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerVolume {
  pub volume: u8,
  pub muted: bool,
  pub fixed: bool
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct HomeTheaterOptions {
  pub night_mode: bool,
  pub enhance_dialog: bool
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatus {
  pub playback_state: PlaybackState,
  pub queue_version: Option<String>,
  pub item_id: Option<String>,
  pub position_millis: i64,
  pub previous_position_millis: i64,
  pub play_modes: PlayModes,
  pub available_playback_actions: AvailablePlaybackActions,
  pub is_ducking: bool
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct PlayModes {
  pub repeat: bool,
  pub repeat_one: bool,
  pub crossfade: bool,
  pub shuffle: bool
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePlaybackActions {
  pub can_skip: bool,
  pub can_skip_back: bool,
  pub can_seek: bool,
  pub can_repeat: bool,
  pub can_repeat_one: bool,
  pub can_crossfade: bool,
  pub can_shuffle: bool,
  pub can_pause: bool,
  pub can_stop: bool
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Favorites {
  pub version: String,
  pub items: Vec<Favorite>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
  pub id: FavoriteId,
  pub name: String,
  pub description: Option<String>,
  pub image_url: Option<String>,
  pub service: Service
}

/// The music service identifier or a pseudo-service identifier in the case
/// of local library.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Service {
  /// The name of the service.
  pub name: String,
  /// The unique identifier for the music service.
  pub id: Option<String>,
  pub images: Vec<String>,
  pub image_url: Option<String>
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

/// The music object identifier for the item in a music service.
/// This identifies the content within a music service, the music service, and
/// the account associated with the content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MusicObjectId {
  pub service_id: Option<String>,
  pub object_id: String,
  pub account_id: Option<String>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Image {
  url: String,
  width: u32,
  height: u32
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Container {
  pub name: Option<String>,
  #[serde(rename = "type")]
  pub type_: Option<String>,
  pub id: Option<MusicObjectId>,
  pub service: Option<Service>,
  pub images: Vec<Image>,
  pub image_url: Option<String>,
  #[serde(default = "Vec::new")]
  pub tags: Vec<Tag>,
  pub book: Option<Book>,
  pub explicit: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Book {
  pub name: String,
  pub author: Author
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Author {
  pub name: String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Narrator {
  pub name: String
}

/// An item in a queue. Used for cloud queue tracks and radio stations that
/// have track-like data for the currently playing content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Item {
  /// The cloud queue itemId for the track.
  /// Only present if the track is from a cloud queue.
  pub id: Option<String>,
  pub track: Track,
  pub deleted: Option<bool>,
  pub policies: Option<Policies>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Policies {}

/// A single music track or audio file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Track {
  #[serde(rename = "type")]
  pub type_: Option<String>,
  pub can_crossfade: Option<bool>,
  pub can_skip: Option<bool>,
  /// The duration of the track, in milliseconds.
  pub duration_millis: Option<i32>,
  /// The unique music service object ID for this track; identifies the
  /// track within the music service from which the track originated.
  pub id: Option<MusicObjectId>,
  /// A URL to an image for the track, for example, an album cover.
  pub image_url: Option<String>,
  pub images: Vec<Image>,
  /// The name of the track.
  pub name: Option<String>,
  pub track_number: Option<u16>,
  pub album: Option<Album>,
  pub artist: Option<Artist>,
  pub author: Option<Author>,
  pub narrator: Option<Narrator>,
  /// The track gain.
  pub replay_gain: Option<f32>,
  #[serde(default = "Vec::new")]
  pub tags: Vec<Tag>,
  pub service: Service,
  pub explicit: bool,
  pub quality: f32
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Album {
  pub name: String,
  pub artist: Option<Artist>,
  pub explicit: bool
}

/// The artist of a track or album.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
  pub name: String,
  pub image_url: Option<String>,
  pub id: Option<MusicObjectId>,
  #[serde(default = "Vec::new")]
  pub tags: Vec<Tag>,
  pub explicit: bool
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatus {
  pub container: Option<Container>,
  pub current_item: Option<Item>,
  pub next_item: Option<Item>,
  /// An unstructured text string describing what is currently playing.
  /// Typically only available for stations that do not have `current_item`
  /// information.
  pub stream_info: Option<String>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistsList {
  pub version: String,
  pub playlists: Vec<Playlist>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
  pub id: PlaylistId,
  pub name: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub track_count: u32
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSummary {
  pub id: PlaylistId,
  pub name: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub tracks: Vec<PlaylistTrack>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistTrack {
  pub name: String,
  pub artist: String,
  pub album: Option<String>
}

pub struct Sonos {
  client: Client,
  integration: Option<IntegrationConfig>,
  integration_path: Option<std::path::PathBuf>,
  tokens: Option<Tokens>,
  tokens_path: Option<std::path::PathBuf>
}

fn from_request_token_error(
  error: RequestTokenError<oauth2::reqwest::HttpClientError, BasicErrorResponse>
) -> Error {
  use RequestTokenError::*;
  match error {
    Request(error) => {
      use oauth2::reqwest::Error::*;
      match error {
        Reqwest(e) => ErrorKind::Request(e).into(),
        Http(_) => unreachable!(),
        Io(e) => ErrorKind::IO(e).into(),
        Other(s) => s.into()
      }
    }
    ServerResponse(e) => format!("{}", e).into(),
    Parse(e, _) => ErrorKind::SerdeJson(e).into(),
    Other(s) => s.into()
  }
}

impl Sonos {
  pub fn is_registered(&self) -> bool { self.integration.is_some() }

  pub fn is_authorized(&self) -> bool { self.tokens.is_some() }

  pub fn set_integration_config(&mut self,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_url: RedirectUrl
  ) -> Result<()> {
    self.integration = Some(
      IntegrationConfig { client_id, client_secret, redirect_url }
    );
    if let Some(path) = &self.integration_path {
      write(path,
        toml::to_string_pretty(self.integration.as_ref().unwrap())?
      )?
    }
    Ok(())
  }

  pub fn authorization_url(&self) -> Result<(Url, CsrfToken)> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        let url = auth
          .authorize_url(CsrfToken::new_random)
          .add_scope(Scope::new("playback-control-all".to_string()))
          .url();
        Ok(url)
      }
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }

  pub fn authorize(&mut self,
    code: AuthorizationCode
  ) -> Result<()> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        let token_result = auth
          .exchange_code(code)
          .request(http_client)
          .map_err(from_request_token_error)
          .chain_err(|| "Failed to exchange code")?;
        let access_token = token_result.access_token().clone();
        if let Some(refresh_token) = token_result.refresh_token() {
          let refresh_token = refresh_token.clone();
          self.tokens = Some(Tokens { access_token, refresh_token });
          if let Some(path) = &self.tokens_path {
            write(path,
              toml::to_string_pretty(&self.tokens.as_ref().unwrap())?
            )?;
          }
          Ok(())
        } else {
          Err("No refresh token received".into())
        }
      }
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }

  fn refresh_token(&mut self) -> Result<&mut Self> {
    match &self.integration {
      Some(integration) => {
        let auth = oauth2(&integration.client_id, &integration.client_secret,
          &integration.redirect_url
        )?;
        match &self.tokens {
          Some(tokens) => {
            let token_response = auth
              .exchange_refresh_token(&tokens.refresh_token)
              .request(http_client)
              .map_err(from_request_token_error)
              .chain_err(|| "Failed to refresh token")?;
            let access_token = token_response.access_token().clone();
            let refresh_token = token_response
              .refresh_token()
              .unwrap_or(&tokens.refresh_token)
              .clone();
            self.tokens = Some(Tokens { access_token, refresh_token });
            if let Some(tokens_path) = &self.tokens_path {
              write(
                tokens_path,
                toml::to_string_pretty(self.tokens.as_ref().unwrap())?
              )?;
            }
            Ok(self)
          }
          None => Err(ErrorKind::TokenRequired.into())
        }
      }
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }

  fn maybe_refresh<B: Fn(&Client) -> RequestBuilder>(&mut self,
    build: B
  ) -> Result<Response> {
    match &self.tokens {
      Some(tokens) => Ok(
        {
          let response = build(&self.client)
            .bearer_auth(tokens.access_token.secret())
            .send()?;
          if response.status() == StatusCode::UNAUTHORIZED {
            self.refresh_token()?;
            build(&self.client)
              .bearer_auth(self.tokens.as_ref().unwrap().access_token.secret())
              .send()?
          } else {
            response
          }
        }
        .error_for_status()?
      ),
      None => Err(ErrorKind::TokenRequired.into())
    }
  }

  /// See Sonos API documentation for [getHouseholds]
  ///
  /// [getHouseholds]: https://developer.sonos.com/reference/control-api/households/
  pub fn get_households(self: &mut Self) -> Result<Vec<Household>> {
    let response =
      self.maybe_refresh(|client| client.get(control_v1!("households")))?;
    let Households { households } = response.json()?;
    Ok(households)
  }

  /// See Sonos API documentation for [getGroups]
  ///
  /// [getGroups]: https://developer.sonos.com/reference/control-api/groups/getgroups/
  pub fn get_groups(self: &mut Self, household: &Household) -> Result<Groups> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("households/{}/groups", household.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [getFavorites]
  ///
  /// [getFavorites]: https://developer.sonos.com/reference/control-api/favorites/getfavorites/
  pub fn get_favorites(self: &mut Self, household: &Household) -> Result<Favorites> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("households/{}/favorites", household.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [getPlaylists]
  ///
  /// [getPlaylists]: https://developer.sonos.com/reference/control-api/playlists/getplaylists/
  pub fn get_playlists(
    self: &mut Self,
    household: &Household
  ) -> Result<PlaylistsList> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("households/{}/playlists", household.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [getPlaylist]
  ///
  /// [getPlaylist]: https://developer.sonos.com/reference/control-api/playlists/getplaylist/
  pub fn get_playlist(
    self: &mut Self,
    household: &Household,
    playlist: &Playlist
  ) -> Result<PlaylistSummary> {
    let mut params = HashMap::new();
    params.insert("playlistId", playlist.id.clone());
    let response = self.maybe_refresh(|client|
      client
        .post(control_v1!("households/{}/playlists/getPlaylist", household.id))
        .json(&params)
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [getPlaybackStatus]
  ///
  /// [getPlaybackStatus]: https://developer.sonos.com/reference/control-api/playback/getplaybackstatus/
  pub fn get_playback_status(self: &mut Self, group: &Group) -> Result<PlaybackStatus> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("groups/{}/playback", group.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [loadLineIn]
  ///
  /// [loadLineIn]: https://developer.sonos.com/reference/control-api/playback/loadlinein/
  pub fn load_line_in(
    self: &mut Self,
    group: &Group,
    player: Option<&Player>,
    play_on_completion: bool
  ) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      device_id: Option<&'a PlayerId>,
      play_on_completion: Option<bool>
    }
    let params = Params {
      device_id: player.map(|player| &player.id),
      play_on_completion: Some(play_on_completion)
    };
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/lineIn", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [getMetadataStatus]
  ///
  /// [getMetadataStatus]: https://developer.sonos.com/reference/control-api/playback-metadata/getmetadatastatus/
  pub fn get_metadata_status(self: &mut Self, group: &Group) -> Result<MetadataStatus> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("groups/{}/playbackMetadata", group.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [loadFavorite]
  ///
  /// [loadFavorite]: https://developer.sonos.com/reference/control-api/favorites/loadfavorite/
  pub fn load_favorite(
    self: &mut Self,
    group: &Group,
    favorite: &Favorite,
    play_on_completion: bool,
    play_modes: Option<&PlayModes>
  ) -> Result<()> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      favorite_id: &'a FavoriteId,
      play_on_completion: bool,
      play_modes: Option<&'a PlayModes>
    }
    let params = Params {
      favorite_id: &favorite.id,
      play_on_completion,
      play_modes
    };
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/favorites", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [loadPlaylist]
  ///
  /// [loadPlaylist]: https://developer.sonos.com/reference/control-api/playlists/loadplaylist/
  pub fn load_playlist(
    self: &mut Self,
    group: &Group,
    playlist: &Playlist,
    play_on_completion: bool,
    play_modes: Option<&PlayModes>
  ) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      playlist_id: &'a PlaylistId,
      play_on_completion: bool,
      play_modes: Option<&'a PlayModes>
    }
    let params = Params {
      playlist_id: &playlist.id,
      play_on_completion,
      play_modes
    };
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playlists", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [getVolume]
  ///
  /// [getVolume]: https://developer.sonos.com/reference/control-api/group-volume/getvolume/
  pub fn get_group_volume(&mut self, group: &Group) -> Result<GroupVolume> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("groups/{}/groupVolume", group.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setVolume]: https://developer.sonos.com/reference/control-api/group-volume/set-volume/
  pub fn set_group_volume(&mut self, group: &Group, volume: u8) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volume", volume);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/groupVolume", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [setRelativeVolume]
  ///
  /// [setRelativeVolume]: https://developer.sonos.com/reference/control-api/group-volume/set-relative-volume/
  pub fn set_relative_group_volume(&mut self,
    group: &Group,
    volume_delta: i8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volumeDelta", volume_delta);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/groupVolume/relative", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [setMute]
  ///
  /// [setMute]: https://developer.sonos.com/reference/control-api/group-volume/set-mute/
  pub fn set_group_mute(&mut self,
    group: &Group,
    muted: bool
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("muted", muted);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/groupVolume/mute", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [play]
  ///
  /// [play]: https://developer.sonos.com/reference/control-api/playback/play/
  pub fn play(&mut self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/play", group.id))
        .header("Content-Type", "application/json")
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [pause]
  ///
  /// [pause]: https://developer.sonos.com/reference/control-api/playback/pause/
  pub fn pause(&mut self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/pause", group.id))
        .header("Content-Type", "application/json")
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [togglePlayPause]
  ///
  /// [togglePlayPause]: https://developer.sonos.com/reference/control-api/playback/toggleplaypause/
  pub fn toggle_play_pause(&mut self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/togglePlayPause", group.id))
        .header("Content-Type", "application/json")
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [skipToNextTrack]
  ///
  /// [skipToNextTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-next-track/
  pub fn skip_to_next_track(&mut self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/skipToNextTrack", group.id))
        .header("Content-Type", "application/json")
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [skipToPreviousTrack]
  ///
  /// [skipToPreviousTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-previous-track/
  pub fn skip_to_previous_track(&mut self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/skipToPreviousTrack", group.id))
        .header("Content-Type", "application/json")
    )?;
    Ok(())
  }
  /// See Sonos API documentation for [seek]
  ///
  /// [seek]: https://developer.sonos.com/reference/control-api/playback/seek/
  pub fn seek(&mut self,
    group: &Group,
    position_millis: u128,
    item_id: Option<&String>
  ) -> Result<()> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      position_millis: u128,
      item_id: Option<&'a String>
    }
    let params = Params {
      position_millis,
      item_id
    };
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/seek", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [seekRelative]
  ///
  /// [seekRelative]: https://developer.sonos.com/reference/control-api/playback/seekrelative/
  pub fn seek_relative(&mut self,
    group: &Group,
    delta_millis: i128,
    item_id: Option<&String>
  ) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      delta_millis: i128,
      item_id: Option<&'a String>
    }
    let params = Params {
      delta_millis,
      item_id
    };
    self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/playback/seekRelative", group.id))
        .json(&params)
    )?;
    Ok(())
  }

  pub fn get_player_volume(&mut self,
    player: &Player
  ) -> Result<PlayerVolume> {
    let response = self.maybe_refresh(|client|
      client.get(control_v1!("players/{}/playerVolume", player.id))
    )?;
    Ok(response.json()?)
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setVolume]: https://developer.sonos.com/reference/control-api/playervolume/setvolume/
  pub fn set_player_volume(&mut self,
    player: &Player,
    volume: u8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volume", volume);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("players/{}/playerVolume", player.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [setRelativeVolume]
  ///
  /// [setRelativeVolume]: https://developer.sonos.com/reference/control-api/playervolume/setrelativevolume/
  pub fn set_relative_player_volume(&mut self,
    player: &Player,
    volume_delta: i8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volumeDelta", volume_delta);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("players/{}/playerVolume/relative", player.id))
        .json(&params)
    )?;
    Ok(())
  }

  /// See Sonos API documentation for [setMute]
  ///
  /// [setMute]: https://developer.sonos.com/reference/control-api/playervolume/setmute/
  pub fn set_player_mute(&mut self,
    player: &Player,
    muted: bool
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("muted", muted);
    self.maybe_refresh(|client|
      client
        .post(control_v1!("players/{}/playerVolume/mute", player.id))
        .json(&params)
    )?;
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub fn load_audio_clip(&mut self,
    player: &Player,
    app_id: &str,
    name: &str,
    clip_type: Option<AudioClipType>,
    priority: Option<Priority>,
    volume: Option<u8>,
    http_authorization: Option<&str>,
    stream_url: Option<&Url>
  ) -> Result<AudioClip> {
    if player.capabilities.contains(&Capability::AudioClip) {
      #[derive(Serialize)]
      #[serde(rename_all = "camelCase")]
      struct Params<'a> {
        app_id: &'a str,
        name: &'a str,
        clip_type: Option<AudioClipType>,
        priority: Option<Priority>,
        volume: Option<u8>,
        http_authorization: Option<&'a str>,
        stream_url: Option<&'a str>
      }
      let params = Params {
        app_id,
        name,
        clip_type,
        priority,
        volume,
        http_authorization,
        stream_url: stream_url.map(|url| url.as_str())
      };
      let response = self.maybe_refresh(|client|
        client
          .post(control_v1!("players/{}/audioClip", player.id))
          .json(&params)
      )?;
      let mut audio_clip: AudioClip = response.json()?;
      audio_clip.player_id = Some(player.id.clone());
      Ok(audio_clip)
    } else {
      Err(ErrorKind::MissingCapability(Capability::AudioClip).into())
    }
  }
  pub fn cancel_audio_clip(&mut self,
    audio_clip: &AudioClip
  ) -> Result<()> {
    if let Some(player_id) = &audio_clip.player_id {
      self.maybe_refresh(|client|
        client.delete(control_v1!(
          "players/{}/audioClip/{}",
          player_id,
          audio_clip.id
        ))
      )?;
      Ok(())
    } else {
      Err(ErrorKind::UnknownPlayerId.into())
    }
  }

  /// See Sonos API documentation for [getOptions]
  ///
  /// [getOptions]: https://developer.sonos.com/reference/control-api/hometheater/getoptions/
  pub fn get_home_theater_options(&mut self,
    player: &Player
  ) -> Result<HomeTheaterOptions> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      let response = self.maybe_refresh(|client|
        client.get(control_v1!("players/{}/homeTheater/options", player.id))
      )?;
      Ok(response.json()?)
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [setOptions]
  ///
  /// [setOptions]: https://developer.sonos.com/reference/control-api/hometheater/setoptions/
  pub fn set_home_theater_options(&mut self,
    player: &Player,
    home_theater_options: &HomeTheaterOptions
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(|client|
        client
          .post(control_v1!("players/{}/homeTheater/options", player.id))
          .json(home_theater_options)
      )?;
      Ok(())
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [setTvPowerState]
  ///
  /// [setTvPowerState]: https://developer.sonos.com/reference/control-api/hometheater/set-tv-power-state/
  pub fn set_tv_power_state(&mut self,
    player: &Player,
    tv_power_state: &TvPowerState
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPowerState) {
      let mut params = HashMap::new();
      params.insert("tvPowerState", tv_power_state);
      self.maybe_refresh(|client|
        client
          .post(control_v1!("players/{}/homeTheater/tvPowerState", player.id))
          .json(&params)
      )?;
      Ok(())
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPowerState).into())
    }
  }

  /// See Sonos API documentation for [loadHomeTheaterPlayback]
  ///
  /// [loadHomeTheaterPlayback]: https://developer.sonos.com/reference/control-api/hometheater/load-home-theater-playback/
  pub fn load_home_theater_playback(&mut self,
    player: &Player
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(|client|
        client
          .post(control_v1!("players/{}/homeTheater", player.id))
          .header("Content-Type", "application/json")
      )?;
      Ok(())
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [modifyGroupMembers]
  ///
  /// [modifyGroupMembers]: https://developer.sonos.com/reference/control-api/groups/modifygroupmembers/
  pub fn modify_group_members(&mut self,
    group: &Group,
    player_ids_to_add: &[&PlayerId],
    player_ids_to_remove: &[&PlayerId]
  ) -> Result<ModifiedGroup> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      player_ids_to_add: &'a [&'a PlayerId],
      player_ids_to_remove: &'a [&'a PlayerId]
    }
    let params = Params {
      player_ids_to_add,
      player_ids_to_remove
    };
    let response = self.maybe_refresh(|client|
      client
        .post(control_v1!("groups/{}/groups/modifyGroupMembers", group.id))
        .json(&params)
    )?;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct GroupInfo {
      group: ModifiedGroup
    }
    let group_info: GroupInfo = response.json()?;
    Ok(group_info.group)
  }
}

impl TryFrom<xdg::BaseDirectories> for Sonos {
  type Error = Error;
  fn try_from(xdg_dirs: xdg::BaseDirectories) -> Result<Self> {
    let integration_path = xdg_dirs.place_config_file("sonos_integration.toml")?;
    let tokens_path = xdg_dirs.place_config_file("sonos_tokens.toml")?;
    match read_to_string(&integration_path).and_then(|s| Ok(toml::from_str(&s)?)) {
      Ok(integration) => {
        match read_to_string(&tokens_path).and_then(|s| Ok(toml::from_str(&s)?)) {
          Ok(tokens) => Ok(Sonos {
            client: Client::new(),
            integration: Some(integration),
            integration_path: Some(integration_path),
            tokens: Some(tokens),
            tokens_path: Some(tokens_path)
          }),
          Err(_) => Ok(Sonos {
            client: Client::new(),
            integration: Some(integration),
            integration_path: Some(integration_path),
            tokens: None,
            tokens_path: Some(tokens_path)
          })
        }
      }
      Err(_) => {
        match read_to_string(&tokens_path).and_then(|s| Ok(toml::from_str(&s)?)) {
          Ok(tokens) => Ok(Sonos {
            client: Client::new(),
            integration: None,
            integration_path: Some(integration_path),
            tokens: Some(tokens),
            tokens_path: Some(tokens_path)
          }),
          Err(_) => Ok(Sonos {
            client: Client::new(),
            integration: None,
            integration_path: Some(integration_path),
            tokens: None,
            tokens_path: Some(tokens_path)
          })
        }
      }
    }
  }
}
