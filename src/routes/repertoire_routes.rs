use super::basic_success;
use crate::check_for_permission;
use crate::util::FileUpload;
use auth::User;
use db::models::*;
use error::{GreaseError, GreaseResult};
use serde_json::{json, Value};
use std::path::PathBuf;

pub fn get_song(id: i32, details: Option<bool>, mut user: User) -> GreaseResult<Value> {
    if details.unwrap_or(false) {
        Song::load_with_data(id, &mut user.conn).map(|song_data| json!(song_data))
    } else {
        Song::load(id, &mut user.conn).map(|song| json!(song))
    }
}

pub fn get_songs(mut user: User) -> GreaseResult<Value> {
    Song::load_all_separate_this_semester(&mut user.conn).map(|(current, other)| {
        json!({
            "current": current,
            "other": other,
        })
    })
}

pub fn new_song((new_song, mut user): (NewSong, User)) -> GreaseResult<Value> {
    Song::create(&new_song, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

pub fn update_song(song_id: i32, (updated_song, mut user): (Song, User)) -> GreaseResult<Value> {
    Song::update(song_id, &updated_song, &mut user.conn).map(|_| basic_success())
}

pub fn delete_song(song_id: i32, mut user: User) -> GreaseResult<Value> {
    Song::delete(song_id, &mut user.conn).map(|_| basic_success())
}

pub fn set_song_as_current(song_id: i32, mut user: User) -> GreaseResult<Value> {
    Song::set_current_status(song_id, true, &mut user.conn).map(|_| basic_success())
}

pub fn set_song_as_not_current(song_id: i32, mut user: User) -> GreaseResult<Value> {
    Song::set_current_status(song_id, false, &mut user.conn).map(|_| basic_success())
}

pub fn get_media_types(mut user: User) -> GreaseResult<Value> {
    MediaType::load_all(&mut user.conn).map(|types| json!(types))
}

pub fn get_song_link(link_id: i32, mut user: User) -> GreaseResult<Value> {
    SongLink::load(link_id, &mut user.conn).map(|link| json!(link))
}

pub fn update_song_link(
    link_id: i32,
    (updated_link, mut user): (SongLinkUpdate, User),
) -> GreaseResult<Value> {
    SongLink::update(link_id, updated_link, &mut user.conn).map(|_| basic_success())
}

pub fn upload_file((file, user): (FileUpload, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-repertoire");
    file.upload().map(|_| basic_success())
}

pub fn new_song_link(
    song_id: i32,
    (new_link, mut user): (NewSongLink, User),
) -> GreaseResult<Value> {
    SongLink::create(song_id, new_link, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

pub fn remove_song_link(link_id: i32, mut user: User) -> GreaseResult<Value> {
    SongLink::delete(link_id, &mut user.conn).map(|_| basic_success())
}

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
