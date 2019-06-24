use db::*;
use crate::error::*;
use util::check_for_music_file;
use pinto::query_builder::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

impl Song {
    pub fn load<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<Song> {
        conn.first(&Self::filter(&format!("id = {}", song_id)), format!("No song with id {}.", song_id))
    }

    pub fn load_with_data<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<SongData> {
        let song = Song::load(song_id, conn)?;
        let song_links = conn.load::<SongLink>(
            SongLink::filter(&format!("song = {}", song_id))
                .order_by("name", Order::Asc)
        )?;

        let mut sorted_links = song_links
            .into_iter()
            .fold(HashMap::new(), |mut map, song_link| {
                let section = map.entry(song_link.type_.clone()).or_insert(Vec::new());
                section.push(song_link);
                map
            });
        let media_types = conn.load::<MediaType>(&MediaType::select_all_in_order("`order`", Order::Asc))?;
        let links = media_types
            .into_iter()
            .map(|media_type| {
                let links = sorted_links.remove(&media_type.name).unwrap_or(Vec::new());

                SongLinkSection {
                    section_name: media_type.name,
                    links,
                }
            })
            .collect::<Vec<SongLinkSection>>();

        if sorted_links.len() > 0 {
            Err(GreaseError::ServerError(format!(
                "song had links with unexpected types: {:?}",
                sorted_links.keys()
            )))
        } else {
            Ok(SongData { song, links })
        }
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Song>> {
        conn.load(&Self::select_all_in_order("title", Order::Asc))
    }

    pub fn load_all_separate_this_semester<C: Connection>(
        conn: &mut C,
    ) -> GreaseResult<(Vec<Song>, Vec<Song>)> {
        let mut all_songs = Song::load_all(conn)?;
        let current_songs = all_songs.drain_filter(|song| song.current).collect();

        Ok((
            current_songs,
            all_songs, // non-current songs
        ))
    }

    pub fn create<C: Connection>(new_song: &NewSong, conn: &mut C) -> GreaseResult<i32> {
        new_song.insert_returning_id(conn)
    }

    pub fn update<C: Connection>(song_id: i32, updated_song: &Song, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("id = {}", song_id))
                .set("title", &to_value(&updated_song.title))
                .set("info", &to_value(&updated_song.info))
                .set("current", &to_value(&updated_song.current))
                .set("key", &to_value(&updated_song.key))
                .set("starting_pitch", &to_value(&updated_song.starting_pitch))
                .set("mode", &to_value(&updated_song.mode)),
            format!("No song with id {}.", song_id),
        )
    }

    pub fn set_current_status<C: Connection>(song_id: i32, is_current: bool, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("id = {}", song_id))
                .set("current", &is_current.to_string()),
            format!("No song with id {}.", song_id),
        )
    }

    pub fn delete<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Self::table_name())
                .filter(&format!("id = {}", song_id)),
            format!("No song with id {}.", song_id),
        )
    }
}

impl SongLink {
    pub fn load<C: Connection>(link_id: i32, conn: &mut C) -> GreaseResult<SongLink> {
        conn.first(&Self::filter(&format!("id = {}", link_id)), format!("no song link with id {}", link_id))
    }

    pub fn load_all_with_types<C: Connection>(conn: &mut C) -> GreaseResult<Vec<(SongLink, MediaType)>> {
        conn.load_as::<SongLinkWithTypeRow, (SongLink, MediaType)>(
            Select::new(Self::table_name())
                .join(MediaType::table_name(), "type", "media_type.name", Join::Inner)
                .fields(SongLinkWithTypeRow::field_names())
                .order_by("song_link.name, `order`", Order::Asc)
        )
    }

    pub fn create<C: Connection>(song_id: i32, new_link: NewSongLink, conn: &mut C) -> GreaseResult<i32> {
        let target = check_for_music_file(
            &utf8_percent_encode(&new_link.target, DEFAULT_ENCODE_SET).to_string(),
        )?;

        conn.insert_returning_id(
            Insert::new(Self::table_name())
                .set("song", &to_value(&song_id))
                .set("type", &to_value(&new_link.type_))
                .set("name", &to_value(&new_link.name))
                .set("target", &to_value(&target))
        )
    }

    pub fn delete<C: Connection>(link_id: i32, conn: &mut C) -> GreaseResult<()> {
        let link = Self::load(link_id, conn)?;
        conn.delete_opt(
            Delete::new(Self::table_name())
                .filter(&format!("id = {}", link_id))
        )?;

        let file_name =
            PathBuf::from_str(&format!("./music/{}", &link.target)).map_err(|_err| {
                GreaseError::ServerError(format!(
                    "invalid file name for link with id {}: {}",
                    link_id, &link.target
                ))
            })?;
        if std::fs::metadata(&file_name).is_ok() {
            std::fs::remove_file(&file_name).map_err(|err| {
                GreaseError::ServerError(format!(
                    "error deleting file from music directory: {}",
                    err
                ))
            })?;
        }

        Ok(())
    }

    pub fn update(link_id: i32, updated_link: SongLinkUpdate, conn: &mut DbConn) -> GreaseResult<()> {
        let old_link = Self::load(link_id, conn)?;
        let new_target = utf8_percent_encode(&updated_link.target, DEFAULT_ENCODE_SET).to_string();

        conn.transaction(|transaction| {
            transaction.update_opt(
                Update::new(Self::table_name())
                    .filter(&format!("id = {}", link_id))
                    .set("name = '{}'", &updated_link.name)
                    .set("target = '{}'", &new_target)
            )?;

            if old_link.target != new_target {
                let old_path =
                    PathBuf::from_str(&format!("./music/{}", old_link.target)).map_err(|_err| {
                        GreaseError::ServerError(format!("invalid file name: {}", old_link.target))
                    })?;
                let new_path =
                    PathBuf::from_str(&format!("./music/{}", new_target)).map_err(|_err| {
                        GreaseError::ServerError(format!("invalid file name: {}", new_target))
                    })?;
                if std::fs::metadata(&old_path).is_ok() {
                    std::fs::rename(&old_path, &new_path).map_err(|err| {
                        GreaseError::ServerError(format!("error renaming link target: {}", err))
                    })?;
                } else {
                    return Err(GreaseError::BadRequest(format!(
                        "Song link '{}' has no associated file. Consider deleting and recreating this link.",
                        old_link.name
                    )));
                }
            }

            Ok(())
        })
    }
}

#[derive(Serialize)]
pub struct SongData {
    pub song: Song,
    pub links: Vec<SongLinkSection>,
}

#[derive(Serialize)]
pub struct SongLinkSection {
    pub section_name: String,
    pub links: Vec<SongLink>,
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
pub struct SongLinkWithTypeRow {
    // song link fields
    pub id: i32,
    pub song: i32,
    #[rename = "type"]
    pub type_: String,
    #[rename = "`song_link`.`name`"]
    pub link_name: String,
    pub target: String,
    // media type fields
    #[rename = "`media_type`.`name`"]
    pub type_name: String,
    pub order: i32,
    pub storage: StorageType,
}

impl Into<(SongLink, MediaType)> for SongLinkWithTypeRow {
    fn into(self) -> (SongLink, MediaType) {
        (
            SongLink {
                id: self.id,
                song: self.song,
                type_: self.type_,
                name: self.link_name,
                target: self.target,
            },
            MediaType {
                name: self.type_name,
                order: self.order,
                storage: self.storage,
            },
        )
    }
}
