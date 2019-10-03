use oauth2::basic::BasicClient;
use oauth2::{AccessToken, AuthorizationCode, AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};
use oauth2::reqwest::http_client;
use reqwest::{Client};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{read_to_string, write};
use std::time::Duration;
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioClipType {
  Chime, Custom
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

#[derive(Debug, Deserialize)]
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
  repeat: bool,
  repeat_one: bool,
  crossfade: bool,
  shuffle: bool
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadFavorite {
  favorite_id: FavoriteId,
  play_on_completion: bool,
  play_modes: Option<PlayModes>
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadPlaylist {
  playlist_id: PlaylistId,
  play_on_completion: bool,
  play_modes: Option<PlayModes>
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Seek {
  position_millis: u128,
  item_id: Option<String>
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadLineIn {
  device_id: Option<PlayerId>,
  play_on_completion: Option<bool>
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
  name: Option<String>,
  #[serde(rename = "type")]
  type_: Option<String>,
  id: Option<MusicObjectId>,
  service: Option<Service>,
  image_url: Option<String>,
  tags: Option<Vec<String>>
}
  
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Item {
  id: Option<String>,
  track: Track,
  deleted: Option<bool>,
  policies: Option<Policies>
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
  tags: Option<Vec<String>>,
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

  /// See Sonos API documentation for [getHouseholds]
  ///
  /// [getHouseholds]: https://developer.sonos.com/reference/control-api/households/
  pub fn get_households(self: &mut Self) -> Result<Vec<Household>> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/households", PREFIX))
          .bearer_auth(access_token.secret())
          .send()?
      )
    }, &|mut response| {
      let households: Households = response.json()?;
      Ok(households.households)
    })
  }

  /// See Sonos API documentation for [getGroups]
  ///
  /// [getGroups]: https://developer.sonos.com/reference/control-api/groups/getgroups/
  pub fn get_groups(self: &mut Self,
    household: &Household
  ) -> Result<Groups> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/households/{}/groups",
                        PREFIX, household.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getFavorites]
  ///
  /// [getFavorites]: https://developer.sonos.com/reference/control-api/favorites/getfavorites/
  pub fn get_favorites(self: &mut Self,
    household: &Household
  ) -> Result<Favorites> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/households/{}/favorites",
                        PREFIX, household.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getPlaylists]
  ///
  /// [getPlaylists]: https://developer.sonos.com/reference/control-api/playlists/getplaylists/
  pub fn get_playlists(self: &mut Self,
    household: &Household
  ) -> Result<PlaylistsList> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/households/{}/playlists",
                        PREFIX, household.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
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
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/households/{}/playlists/getPlaylist",
                         PREFIX, household.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [getPlaybackStatus]
  ///
  /// [getPlaybackStatus]: https://developer.sonos.com/reference/control-api/playback/getplaybackstatus/
  pub fn get_playback_status(self: &mut Self,
    group: &Group
  ) -> Result<PlaybackStatus> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/groups/{}/playback",
                        PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [loadLineIn]
  ///
  /// [loadLineIn]: https://developer.sonos.com/reference/control-api/playback/loadlinein/
  pub fn load_line_in(self: &mut Self,
    group: &Group, player: Option<&Player>, play_on_completion: bool
  ) -> Result<()> {
    let params = LoadLineIn {
      device_id: player.map(|player| player.id.clone()),
      play_on_completion: Some(play_on_completion)
    };
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/lineIn", PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [getMetadataStatus]
  ///
  /// [getMetadataStatus]: https://developer.sonos.com/reference/control-api/playback-metadata/getmetadatastatus/
  pub fn get_metadata_status(self: &mut Self,
    group: &Group
  ) -> Result<MetadataStatus> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/groups/{}/playbackMetadata",
                        PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [loadFavorite]
  ///
  /// [loadFavorite]: https://developer.sonos.com/reference/control-api/favorites/loadfavorite/
  pub fn load_favorite(self: &mut Self,
    group: &Group,
    favorite: &Favorite,
    play_on_completion: bool,
    play_modes: Option<PlayModes>
  ) -> Result<()> {
    let params = LoadFavorite {
      favorite_id: favorite.id.clone(),
      play_on_completion, play_modes
    };
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/favorites",
                         PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [loadPlaylist]
  ///
  /// [loadPlaylist]: https://developer.sonos.com/reference/control-api/playlists/loadplaylist/
  pub fn load_playlist(self: &mut Self,
    group: &Group,
    playlist: &Playlist,
    play_on_completion: bool,
    play_modes: Option<PlayModes>
  ) -> Result<()> {
    let params = LoadPlaylist {
      playlist_id: playlist.id.clone(),
      play_on_completion, play_modes
    };
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playlists", PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [getVolume]
  ///
  /// [getVolume]: https://developer.sonos.com/reference/control-api/group-volume/getvolume/
  pub fn get_group_volume(self: &mut Self,
    group: &Group
  ) -> Result<GroupVolume> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .get(&format!("{}/groups/{}/groupVolume", PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .send()?
      )
    }, &|mut response| Ok(response.json()?)
    )
  }

  /// See Sonos API documentation for [setVolume]
  ///
  /// [setVolume]: https://developer.sonos.com/reference/control-api/group-volume/set-volume/
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
          .post(&format!("{}/groups/{}/groupVolume", PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [play]
  ///
  /// [play]: https://developer.sonos.com/reference/control-api/playback/play/
  pub fn play(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/play", PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [pause]
  ///
  /// [pause]: https://developer.sonos.com/reference/control-api/playback/pause/
  pub fn pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/pause", PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [togglePlayPause]
  ///
  /// [togglePlayPause]: https://developer.sonos.com/reference/control-api/playback/toggleplaypause/
  pub fn toggle_play_pause(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/togglePlayPause",
                         PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }

  /// See Sonos API documentation for [skipToNextTrack]
  ///
  /// [skipToNextTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-next-track/
  pub fn skip_to_next_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/skipToNextTrack",
                         PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  /// See Sonos API documentation for [skipToPreviousTrack]
  ///
  /// [skipToPreviousTrack]: https://developer.sonos.com/reference/control-api/playback/skip-to-previous-track/
  pub fn skip_to_previous_track(self: &mut Self,
    group: &Group
  ) -> Result<()> {
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/skipToPreviousTrack",
                         PREFIX, group.id))
          .bearer_auth(access_token.secret()).send()?
      )
    }, &|_response| Ok(())
    )
  }
  /// See Sonos API documentation for [seek]
  ///
  /// [seek]: https://developer.sonos.com/reference/control-api/playback/seek/
  pub fn seek(self: &mut Self,
    group: &Group, position: &Duration, item_id: &Option<String>
  ) -> Result<()> {
    let params = Seek {
      position_millis: position.as_millis(),
      item_id: item_id.clone()
    };
    self.maybe_refresh(&|access_token| {
      let client = Client::new();
      Ok(
        client
          .post(&format!("{}/groups/{}/playback/seek", PREFIX, group.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
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
          .get(&format!("{}/players/{}/playerVolume",
                        PREFIX, player.id))
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
          .post(&format!("{}/players/{}/playerVolume", PREFIX, player.id))
          .bearer_auth(access_token.secret())
          .json(&params)
          .send()?
      )
    }, &|_response| Ok(())
    )
  }
  pub fn load_audio_clip(self: &mut Self,
    player: &Player, app_id: &str, name: &str,
    clip_type: Option<&AudioClipType>, priority: Option<&Priority>,
    volume: Option<u8>,
    http_authorization: Option<&str>, stream_url: Option<&Url>
  ) -> Result<AudioClip> {
    if player.capabilities.contains(&Capability::AudioClip) {
      let mut params = HashMap::new();
      params.insert("appId", app_id.to_string());
      params.insert("name", name.to_string());
      if let Some(clip_type) = clip_type {
        params.insert("clipType", serde_json::to_string(clip_type)?);
      }
      if let Some(priority) = priority {
        params.insert("priority", serde_json::to_string(priority)?);
      }
      if let Some(volume) = volume {
        params.insert("volume", volume.to_string());
      }
      if let Some(stream_url) = stream_url {
        params.insert("streamUrl", stream_url.to_string());
      }
      if let Some(http_authorization) = http_authorization {
        params.insert("httpAuthorization", http_authorization.to_string());
      }
      self.maybe_refresh(&|access_token| {
        let client = Client::new();
        Ok(
          client
            .post(&format!("{}/players/{}/audioClip", PREFIX, player.id))
            .bearer_auth(access_token.secret())
            .json(&params)
            .send()?
        )
      }, &|mut response| {
        let mut audio_clip: AudioClip = response.json()?;
        audio_clip.player_id = Some(player.id.clone());
        Ok(audio_clip)
      })
    } else {
      Err(ErrorKind::MissingCapability(Capability::AudioClip).into())
    }
  }
  pub fn cancel_audio_clip(self: &mut Self,
    audio_clip: &AudioClip
  ) -> Result<()> {
    if let Some(player_id) = &audio_clip.player_id {
      self.maybe_refresh(&|access_token| {
        let client = Client::new();
        Ok(
          client
            .delete(&format!("{}/players/{}/audioClip/{}",
                             PREFIX, player_id, audio_clip.id))
            .bearer_auth(access_token.secret())
            .send()?
        )
      }, &|_response| Ok(())
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
      self.maybe_refresh(&|access_token| {
        let client = Client::new();
        Ok(
          client
            .get(&format!("{}/players/{}/homeTheater/options", PREFIX, player.id))
            .bearer_auth(access_token.secret())
            .send()?
        )
      }, &|mut response| Ok(response.json()?)
      )
    } else {
      Err(ErrorKind::MissingCapability(Capability::HtPlayback).into())
    }
  }

  /// See Sonos API documentation for [loadHomeTheaterPlayback]
  ///
  /// [loadHomeTheaterPlayback]: https://developer.sonos.com/reference/control-api/hometheater/load-home-theater-playback/
  pub fn load_home_theater_playback(self: &mut Self,
    player: &Player
  ) -> Result<()> {
    if player.capabilities.contains(&Capability::HtPlayback) {
      self.maybe_refresh(&|access_token| {
        let client = Client::new();
        Ok(
          client
            .post(&format!("{}/players/{}/homeTheater", PREFIX, player.id))
            .bearer_auth(access_token.secret())
            .send()?
        )
      }, &|_response| Ok(())
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
