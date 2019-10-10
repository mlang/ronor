use oauth2::basic::BasicClient;
use oauth2::{AccessToken, AuthorizationCode, AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};
use oauth2::reqwest::http_client;
use reqwest::{Client};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{read_to_string, write};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use url::Url;

#[macro_use]
extern crate error_chain;

error_chain!{
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
const PREFIX: &str = "https://api.ws.sonos.com/control/api/v1";

fn oauth2(
  client_id: &ClientId, client_secret: &ClientSecret, redirect_url: &RedirectUrl
) -> Result<BasicClient> {
  Ok(BasicClient::new(client_id.clone(), Some(client_secret.clone()),
      AuthUrl::new(Url::parse(AUTH_URL)?),
      Some(TokenUrl::new(Url::parse(TOKEN_URL)?))
    ).set_redirect_url(redirect_url.clone())
  )
}

macro_rules! ids {
  ($name:ident) => {
    #[derive (Clone, Debug, Deserialize, Serialize)]
    pub struct $name(String);

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }
  };
  ($name:ident,$($more:ident),+) => { ids!($name); ids!($($more),+); }
}

ids!(HouseholdId, GroupId, PlayerId, FavoriteId, PlaylistId, AudioClipId);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioClipType {
  Chime, Custom
}

impl FromStr for AudioClipType {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Chime" => Ok(AudioClipType::Chime),
      "Custom" => Ok(AudioClipType::Custom),
      _        => Err("no match".into())
    }
  }
}

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
pub enum PlaybackState {
  #[serde(rename = "PLAYBACK_STATE_IDLE")]
  Idle,
  #[serde(rename = "PLAYBACK_STATE_PAUSED")]
  Paused,
  #[serde(rename = "PLAYBACK_STATE_BUFFERING")]
  Buffering,
  #[serde(rename = "PLAYBACK_STATE_PLAYING")]
  Playing
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
  Low, High
}

impl FromStr for Priority {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self> {
    match s {
      "Low"  => Ok(Priority::Low),
      "High" => Ok(Priority::High),
      _      => Err("no match".into())
    }
  }
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
  id: AudioClipId,
  priority: Option<Priority>,
  status: Option<String>,
  #[serde(skip)]
  player_id: Option<PlayerId>
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupVolume {
  volume: u8,
  muted: bool,
  fixed: bool
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerVolume {
  volume: u8,
  muted: bool,
  fixed: bool
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
  can_skip: bool,
  can_skip_back: bool,
  can_seek: bool,
  can_repeat: bool,
  can_repeat_one: bool,
  can_crossfade: bool,
  can_shuffle: bool,
  can_pause: bool,
  can_stop: bool
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Service {
  pub name: String,
  pub id: Option<String>,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MusicObjectId {
  service_id: Option<String>,
  object_id: String,
  account_id: Option<String>
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
  pub image_url: Option<String>,
  pub tags: Option<Vec<String>>
}
  
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Item {
  pub id: Option<String>,
  pub track: Track,
  pub deleted: Option<bool>,
  pub policies: Option<Policies>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Policies {
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
  pub can_crossfade: Option<bool>,
  pub can_skip: Option<bool>,
  pub duration_millis: Option<i32>,
  pub id: Option<MusicObjectId>,
  pub image_url: Option<String>,
  pub name: Option<String>,
  pub replay_gain: Option<f32>,
  pub tags: Option<Vec<String>>,
  pub service: Service
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatus {
  pub container: Option<Container>,
  pub current_item: Option<Item>,
  pub next_item: Option<Item>,
  pub stream_info: Option<String>,
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

impl Sonos {
  pub fn is_registered(self: &Self) -> bool { self.integration.is_some() }

  pub fn is_authorized(self: &Self) -> bool { self.tokens.is_some() }

  pub fn set_integration_config(self: &mut Self,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_url: RedirectUrl
  ) -> Result<()> {
    self.integration = Some(IntegrationConfig {
        client_id, client_secret, redirect_url
    });
    if let Some(path) = &self.integration_path {
      write(path, toml::to_string_pretty(self.integration.as_ref().unwrap())?)?
    }
    Ok(())
  }

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
          if let Some(path) = &self.tokens_path {
            write(path, toml::to_string_pretty(&self.tokens.as_ref().unwrap())?)?;
          }
          Ok(())
        } else {
          Err("No refresh token received".into())
        }
      },
      None => Err(ErrorKind::IntegrationRequired.into())
    }
  }
  
  fn refresh_token(self: &mut Self) -> Result<&mut Self> {
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

  fn maybe_refresh<P, C, T>(self: &mut Self,
    prepare: P, convert: C
  ) -> Result<T> where P: Fn(&Client) -> reqwest::RequestBuilder,
                       C: FnOnce(reqwest::Response) -> Result<T>
  {
    match &self.tokens {
      Some(tokens) => convert({
        let response = prepare(&self.client)
          .bearer_auth(tokens.access_token.secret())
          .send()?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
          self.refresh_token()?;
          prepare(&self.client)
            .bearer_auth(self.tokens.as_ref().unwrap().access_token.secret())
            .send()?
        } else {
          response
        }
      }.error_for_status()?),
      None => Err(ErrorKind::TokenRequired.into())
    }
  }

  /// See Sonos API documentation for [getHouseholds]
  ///
  /// [getHouseholds]: https://developer.sonos.com/reference/control-api/households/
  pub fn get_households(self: &mut Self) -> Result<Vec<Household>> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/households", PREFIX)),
      |mut response| {
        let households: Households = response.json()?;
        Ok(households.households)
      }
    )
  }

  /// See Sonos API documentation for [getGroups]
  ///
  /// [getGroups]: https://developer.sonos.com/reference/control-api/groups/getgroups/
  pub fn get_groups(self: &mut Self,
    household: &Household
  ) -> Result<Groups> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/households/{}/groups", PREFIX, household.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getFavorites]
  ///
  /// [getFavorites]: https://developer.sonos.com/reference/control-api/favorites/getfavorites/
  pub fn get_favorites(self: &mut Self,
    household: &Household
  ) -> Result<Favorites> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/households/{}/favorites", PREFIX, household.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getPlaylists]
  ///
  /// [getPlaylists]: https://developer.sonos.com/reference/control-api/playlists/getplaylists/
  pub fn get_playlists(self: &mut Self,
    household: &Household
  ) -> Result<PlaylistsList> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/households/{}/playlists", PREFIX, household.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getPlaylist]
  ///
  /// [getPlaylist]: https://developer.sonos.com/reference/control-api/playlists/getplaylist/
  pub fn get_playlist(self: &mut Self,
    household: &Household, playlist: &Playlist
  ) -> Result<PlaylistSummary> {
    let mut params = HashMap::new();
    params.insert("playlistId", playlist.id.clone());
    self.maybe_refresh(
      |client| client.post(&format!("{}/households/{}/playlists/getPlaylist", PREFIX, household.id))
                     .json(&params),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getPlaybackStatus]
  ///
  /// [getPlaybackStatus]: https://developer.sonos.com/reference/control-api/playback/getplaybackstatus/
  pub fn get_playback_status(self: &mut Self,
    group: &Group
  ) -> Result<PlaybackStatus> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/groups/{}/playback", PREFIX, group.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [loadLineIn]
  ///
  /// [loadLineIn]: https://developer.sonos.com/reference/control-api/playback/loadlinein/
  pub fn load_line_in(self: &mut Self,
    group: &Group, player: Option<&Player>, play_on_completion: bool
  ) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      device_id: Option<&'a PlayerId>,
      play_on_completion: Option<bool>
    }
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playback/lineIn", PREFIX, group.id))
                     .json(&Params {
                       device_id: player.map(|player| &player.id),
                       play_on_completion: Some(play_on_completion)
                     }),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [getMetadataStatus]
  ///
  /// [getMetadataStatus]: https://developer.sonos.com/reference/control-api/playback-metadata/getmetadatastatus/
  pub fn get_metadata_status(self: &mut Self,
    group: &Group
  ) -> Result<MetadataStatus> {
    self.maybe_refresh(|client| {
      client
          .get(&format!("{}/groups/{}/playbackMetadata", PREFIX, group.id))
    }, |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [loadFavorite]
  ///
  /// [loadFavorite]: https://developer.sonos.com/reference/control-api/favorites/loadfavorite/
  pub fn load_favorite(self: &mut Self,
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
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/favorites", PREFIX, group.id))
                     .json(&Params {
                       favorite_id: &favorite.id,
                       play_on_completion, play_modes
                     }),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [loadPlaylist]
  ///
  /// [loadPlaylist]: https://developer.sonos.com/reference/control-api/playlists/loadplaylist/
  pub fn load_playlist(self: &mut Self,
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
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playlists", PREFIX, group.id))
                     .json(&Params {
                       playlist_id: &playlist.id,
                       play_on_completion, play_modes
                     }),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [getVolume]
  ///
  /// [getVolume]: https://developer.sonos.com/reference/control-api/group-volume/getvolume/
  pub fn get_group_volume(self: &mut Self,
    group: &Group
  ) -> Result<GroupVolume> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/groups/{}/groupVolume", PREFIX, group.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setVolume]: https://developer.sonos.com/reference/control-api/group-volume/set-volume/
  pub fn set_group_volume(self: &mut Self,
    group: &Group,
    volume: u8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volume", volume);
    self.maybe_refresh(|client| {
      client.post(&format!("{}/groups/{}/groupVolume", PREFIX, group.id))
        .json(&params)
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [setRelativeVolume]
  ///
  /// [setRelativeVolume]: https://developer.sonos.com/reference/control-api/group-volume/set-relative-volume/
  pub fn set_relative_group_volume(self: &mut Self,
    group: &Group,
    volume_delta: i8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volumeDelta", volume_delta);
    self.maybe_refresh(|client| {
      client.post(&format!("{}/groups/{}/groupVolume/relative", PREFIX, group.id))
        .json(&params)
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [setMute]
  ///
  /// [setMute]: https://developer.sonos.com/reference/control-api/group-volume/set-mute/
  pub fn set_group_mute(self: &mut Self,
    group: &Group,
    muted: bool
  ) -> Result<()> {
   let mut params = HashMap::new();
    params.insert("muted", muted);
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/groupVolume/mute", PREFIX, group.id))
                     .json(&params),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [play]
  ///
  /// [play]: https://developer.sonos.com/reference/control-api/playback/play/
  pub fn play(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client| {
      client.post(&format!("{}/groups/{}/playback/play", PREFIX, group.id))
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [pause]
  ///
  /// [pause]: https://developer.sonos.com/reference/control-api/playback/pause/
  pub fn pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client| {
      client.post(&format!("{}/groups/{}/playback/pause", PREFIX, group.id))
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [togglePlayPause]
  ///
  /// [togglePlayPause]: https://developer.sonos.com/reference/control-api/playback/toggleplaypause/
  pub fn toggle_play_pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(|client| {
      client.post(&format!("{}/groups/{}/playback/togglePlayPause", PREFIX, group.id))
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [skipToNextTrack]
  ///
  /// [skipToNextTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-next-track/
  pub fn skip_to_next_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playback/skipToNextTrack", PREFIX, group.id)),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [skipToPreviousTrack]
  ///
  /// [skipToPreviousTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-previous-track/
  pub fn skip_to_previous_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playback/skipToPreviousTrack", PREFIX, group.id)),
      |_response| Ok(())
    )
  }
  /// See Sonos API documentation for [seek]
  ///
  /// [seek]: https://developer.sonos.com/reference/control-api/playback/seek/
  pub fn seek(self: &mut Self,
    group: &Group, position_millis: u128, item_id: Option<&String>
  ) -> Result<()> {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      position_millis: u128,
      item_id: Option<&'a String>
    }
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playback/seek", PREFIX, group.id))
                     .json(&Params { position_millis, item_id }),
      |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [seekRelative]
  ///
  /// [seekRelative]: https://developer.sonos.com/reference/control-api/playback/seekrelative/
  pub fn seek_relative(self: &mut Self,
    group: &Group, delta_millis: i128, item_id: Option<&String>
  ) -> Result<()> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
      delta_millis: i128,
      item_id: Option<&'a String>
    }
    self.maybe_refresh(
      |client| client.post(&format!("{}/groups/{}/playback/seekRelative", PREFIX, group.id))
                     .json(&Params { delta_millis, item_id }),
      |_response| Ok(())
    )
  }

  pub fn get_player_volume(self: &mut Self,
    player: &Player
  ) -> Result<PlayerVolume> {
    self.maybe_refresh(
      |client| client.get(&format!("{}/players/{}/playerVolume", PREFIX, player.id)),
      |mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setVolume]: https://developer.sonos.com/reference/control-api/playervolume/setvolume/
  pub fn set_player_volume(self: &mut Self,
    player: &Player,
    volume: u8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volume", volume);
    self.maybe_refresh(|client| {
      client.post(&format!("{}/players/{}/playerVolume", PREFIX, player.id))
        .json(&params)
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [setRelativeVolume]
  ///
  /// [setRelativeVolume]: https://developer.sonos.com/reference/control-api/playervolume/setrelativevolume/
  pub fn set_relative_player_volume(self: &mut Self,
    player: &Player,
    volume_delta: i8
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("volumeDelta", volume_delta);
    self.maybe_refresh(|client| {
      client.post(&format!("{}/players/{}/playerVolume/relative", PREFIX, player.id))
        .json(&params)
    }, |_response| Ok(())
    )
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setMute]: https://developer.sonos.com/reference/control-api/playervolume/setmute/
  pub fn set_player_mute(self: &mut Self,
    player: &Player,
    muted: bool
  ) -> Result<()> {
    let mut params = HashMap::new();
    params.insert("muted", muted);
    self.maybe_refresh(|client| {
      client.post(&format!("{}/players/{}/playerVolume/mute", PREFIX, player.id))
        .json(&params)
    }, |_response| Ok(())
    )
  }

  pub fn load_audio_clip(self: &mut Self,
    player: &Player, app_id: &str, name: &str,
    clip_type: Option<AudioClipType>, priority: Option<Priority>,
    volume: Option<u8>,
    http_authorization: Option<&str>, stream_url: Option<&Url>
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
        app_id, name, clip_type, priority, volume, http_authorization,
        stream_url: stream_url.map(|url| url.as_str())
      };
      self.maybe_refresh(
        |client| client.post(&format!("{}/players/{}/audioClip", PREFIX, player.id))
                       .json(&params),
        |mut response| {
          let mut audio_clip: AudioClip = response.json()?;
          audio_clip.player_id = Some(player.id.clone());
          Ok(audio_clip)
        }
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::AudioClip).into())
    }
  }
  pub fn cancel_audio_clip(self: &mut Self,
    audio_clip: &AudioClip
  ) -> Result<()> {
    if let Some(player_id) = &audio_clip.player_id {
      self.maybe_refresh(
        |client| client.delete(&format!("{}/players/{}/audioClip/{}", PREFIX, player_id, audio_clip.id)),
        |_response| Ok(())
      )
    } else {
      Err(ErrorKind::UnknownPlayerId.into())
    }
  }

  /// See Sonos API documentation for [getOptions]
  ///
  /// [getOptions]: https://developer.sonos.com/reference/control-api/hometheater/getoptions/
  pub fn get_home_theater_options(self: &mut Self,
    player: &Player
  ) -> Result<HomeTheaterOptions> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(
        |client| client.get(&format!("{}/players/{}/homeTheater/options", PREFIX, player.id)),
        |mut response| Ok(response.json()?)
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [setOptions]
  ///
  /// [setOptions]: https://developer.sonos.com/reference/control-api/hometheater/setoptions/
  pub fn set_home_theater_options(self: &mut Self,
    player: &Player, home_theater_options: &HomeTheaterOptions
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(
        |client| client.post(&format!("{}/players/{}/homeTheater/options", PREFIX, player.id))
                       .json(home_theater_options),
        |_response| Ok(())
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [setTvPowerState]
  ///
  /// [setTvPowerState]: https://developer.sonos.com/reference/control-api/hometheater/set-tv-power-state/
  pub fn set_tv_power_state(self: &mut Self,
    player: &Player, tv_power_state: &TvPowerState
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPowerState) {
      let mut params = HashMap::new();
      params.insert("tvPowerState", tv_power_state);
      self.maybe_refresh(|client| {
        client.post(&format!("{}/players/{}/homeTheater/tvPowerState", PREFIX, player.id))
          .json(&params)
      }, |_response| Ok(())
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPowerState).into())
    }
  }

  /// See Sonos API documentation for [loadHomeTheaterPlayback]
  ///
  /// [loadHomeTheaterPlayback]: https://developer.sonos.com/reference/control-api/hometheater/load-home-theater-playback/
  pub fn load_home_theater_playback(self: &mut Self,
    player: &Player
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(|client| {
        client.post(&format!("{}/players/{}/homeTheater", PREFIX, player.id))
      }, |_response| Ok(())
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
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
          client: Client::new(),
          integration: Some(integration),
          integration_path: Some(integration_config_path),
          tokens: Some(tokens),
          tokens_path: Some(tokens_config_path)
        }),
        Err(_) => Ok(Sonos {
          client: Client::new(),
          integration: Some(integration),
          integration_path: Some(integration_config_path),
          tokens: None,
          tokens_path: Some(tokens_config_path)
        })
      },
      Err(_) => match read_to_string(&tokens_config_path).and_then(|s| Ok(toml::from_str(&s)?)) {
        Ok(tokens) => Ok(Sonos {
          client: Client::new(),
          integration: None,
          integration_path: Some(integration_config_path),
          tokens: Some(tokens),
          tokens_path: Some(tokens_config_path)
        }),
        Err(_) => Ok(Sonos {
          client: Client::new(),
          integration: None,
          integration_path: Some(integration_config_path),
          tokens: None,
          tokens_path: Some(tokens_config_path)
        })
      }
    }
  }
}
