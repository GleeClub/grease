//! All repertoire-focused routes.

use super::basic_success;
use crate::check_for_permission;
use crate::util::FileUpload;
use auth::User;
use db::schema::StorageType;
use db::*;
use error::{GreaseError, GreaseResult};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::collections::HashMap;

/// Get a single song.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Query Parameters:
///   * details: boolean (*optional*) - Whether to include the song's links
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// If `full = true`, then the format from
/// [load_with_data](crate::db::models::Song#method.load_with_data)
/// will be returned. Otherwise, a simple [Song](crate::db::models::Song)
/// will be returned.
pub fn get_song(id: i32, params: HashMap<String, bool>, mut user: User) -> GreaseResult<Value> {
    if params.get("full").unwrap_or(false) {
        Song::load_with_data(id, &mut user.conn).map(|song_data| json!(song_data))
    } else {
        Song::load(id, &mut user.conn).map(|song| json!(song))
    }
}

/// Get the entire club's repertoire.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [Song](crate::db::models::Song)s ordered alphabetically
/// by title.
pub fn get_songs(mut user: User) -> GreaseResult<Value> {
    Song::load_all(&mut user.conn).map(|songs| json!(songs))
}

/// Create a new song.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
///
/// ## Input Format:
///
/// Expects a [NewSong](crate::db::models::NewSong).
pub fn new_song(new_song: NewSong, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    Song::create(&new_song, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

/// Update a song from the repertoire.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
///
/// ## Input Format:
///
/// Expects a [SongUpdate](crate::db::models::SongUpdate).
pub fn update_song(
    song_id: i32,
    (updated_song, mut user): (SongUpdate, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    Song::update(song_id, &updated_song, &mut user.conn).map(|_| basic_success())
}

/// Delete a song from the repertoire.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
pub fn delete_song(song_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    Song::delete(song_id, &mut user.conn).map(|_| basic_success())
}

/// Add a song to the current semester's repertoire.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
pub fn set_song_as_current(song_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    Song::set_current_status(song_id, true, &mut user.conn).map(|_| basic_success())
}

/// Remove a song from the current semester's repertoire.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
pub fn set_song_as_not_current(song_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    Song::set_current_status(song_id, false, &mut user.conn).map(|_| basic_success())
}

/// Get all of the media types available.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [MediaType](crate::db::models::MediaType)s.
pub fn get_media_types(mut user: User) -> GreaseResult<Value> {
    MediaType::load_all(&mut user.conn).map(|types| json!(types))
}

/// Get a single link belonging to a song.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song link
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a [SongLink](crate::db::models::SongLink).
pub fn get_song_link(link_id: i32, mut user: User) -> GreaseResult<Value> {
    SongLink::load(link_id, &mut user.conn).map(|link| json!(link))
}

/// Update a link belonging to a song.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song link
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
///
/// ## Input Format:
///
/// Expects a [SongLinkUpdate](crate::db::models::SongLinkUpdate).
pub fn update_song_link(
    link_id: i32,
    (updated_link, mut user): (SongLinkUpdate, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    SongLink::update(link_id, updated_link, &mut user.conn).map(|_| basic_success())
}

/// Upload a file for a song link to refer to.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
///
/// ## Input Format:
///
/// Expects a [FileUpload](crate::util::FileUpload).
pub fn upload_file((file, user): (FileUpload, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    file.upload().map(|_| basic_success())
}

/// Create a new link belonging to a song.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
///
/// ## Input Format:
///
/// Expects a [NewSongLink](crate::db::models::NewSongLink).
pub fn new_song_link(
    song_id: i32,
    (new_link, mut user): (NewSongLink, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    SongLink::create(song_id, new_link, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

/// Remove a song link belonging to a song.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the song link
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
pub fn remove_song_link(link_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    SongLink::delete(link_id, &mut user.conn).map(|_| basic_success())
}

/// Remove song files that aren't pointed to by song links.
///
/// If you don't pass `confirm=true`, then this returns a list of the file names
/// that are dangling. If you do confirm, this endpoint will delete those files.
///
/// ## Query Parameters:
///   * confirm: boolean (*optional*) - Confirm the deletion of the dangling files
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-repertoire" generally.
pub fn cleanup_song_files(confirm: Option<bool>, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    let all_music_files = glob::glob("./music/*")
        .map_err(|err| {
            GreaseError::ServerError(format!(
                "couldn't index through music file directory: {}",
                err
            ))
        })?
        .map(|path| {
            let path: PathBuf = path.map_err(|err| {
                GreaseError::ServerError(format!(
                    "error reading filename in music directory: {}",
                    err
                ))
            })?;
            let file_name = path.file_name().ok_or(GreaseError::ServerError(
                "one of the files in the music directory had no name".to_owned(),
            ))?;
            Ok(file_name.to_string_lossy().to_string())
        })
        .collect::<GreaseResult<Vec<String>>>()?;
    let all_song_links_with_types = SongLink::load_all_with_types(&mut user.conn)?;
    let dangling_files = all_music_files
        .iter()
        .filter(|music_file| {
            !all_song_links_with_types
                .iter()
                .any(|(song_link, media_type)| {
                    media_type.storage == StorageType::Local && &&song_link.target == music_file
                })
        })
        .collect::<Vec<&String>>();

    if confirm.unwrap_or(false) {
        for file in dangling_files {
            let path = format!("./music/{}", file);
            std::fs::remove_file(path).map_err(|err| {
                GreaseError::ServerError(format!(
                    "error deleting music file named {}: {}",
                    file, err
                ))
            })?;
        }
        Ok(basic_success())
    } else {
        Ok(json!({
            "dangling_files": dangling_files,
        }))
    }
}
