use crate::db::traits::*;
use crate::error::*;
use crate::util::check_for_music_file;
use db::models::*;
use mysql::Conn;
use mysql_enum::mysql::prelude::ToValue;
use pinto::query_builder::{self, Join, Order};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

impl Song {
    pub fn load(song_id: i32, conn: &mut Conn) -> GreaseResult<Song> {
        Song::first(
            &format!("id = {}", song_id),
            conn,
            format!("no song with id {}", song_id),
        )
    }

    pub fn load_with_data(song_id: i32, conn: &mut Conn) -> GreaseResult<SongData> {
        let song = Song::load(song_id, conn)?;
        let song_links = {
            let query = query_builder::select(SongLink::table_name())
                .fields(SongLink::field_names())
                .filter(&format!("song = {}", song_id))
                .order_by("name", Order::Asc)
                .build();
            crate::db::load::<SongLink, _>(&query, conn)?
        };
        let mut sorted_links = song_links
            .into_iter()
            .fold(HashMap::new(), |mut map, song_link| {
                let section = map.entry(song_link.type_.clone()).or_insert(Vec::new());
                section.push(song_link);
                map
            });
        let media_types = MediaType::query_all_in_order(vec![("`order`", Order::Asc)], conn)?;
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

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<Song>> {
        Song::query_all_in_order(vec![("title", Order::Asc)], conn)
    }

    pub fn load_all_separate_this_semester(
        conn: &mut Conn,
    ) -> GreaseResult<(Vec<Song>, Vec<Song>)> {
        let mut all_songs = Song::load_all(conn)?;
        let current_songs = all_songs.drain_filter(|song| song.current).collect();

        Ok((
            current_songs,
            all_songs, // non-current songs
        ))
    }

    pub fn create(new_song: &NewSong, conn: &mut Conn) -> GreaseResult<i32> {
        new_song.insert_returning_id("id", conn)
    }

    pub fn update(song_id: i32, updated_song: &Song, conn: &mut Conn) -> GreaseResult<()> {
        let key = if let Some(ref key) = &updated_song.key {
            format!("'{}'", key)
        } else {
            "NULL".to_owned()
        };
        let starting_pitch = if let Some(ref starting_pitch) = &updated_song.starting_pitch {
            format!("'{}'", starting_pitch)
        } else {
            "NULL".to_owned()
        };
        let mode = if let Some(ref mode) = &updated_song.mode {
            format!("'{}'", mode)
        } else {
            "NULL".to_owned()
        };
        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", song_id))
            .set("title", &updated_song.title)
            .set("info", &updated_song.info.to_value().as_sql(false))
            .set("current", &updated_song.current.to_string())
            .set("key", &key)
            .set("starting_pitch", &starting_pitch)
            .set("mode", &mode)
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn set_current_status(song_id: i32, is_current: bool, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", song_id))
            .set("current", &is_current.to_string())
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(song_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("id = {}", song_id))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl SongLink {
    pub fn load(link_id: i32, conn: &mut Conn) -> GreaseResult<SongLink> {
        SongLink::first(
            &format!("id = {}", link_id),
            conn,
            format!("no song link with id {}", link_id),
        )
    }

    pub fn load_all_with_types(conn: &mut Conn) -> GreaseResult<Vec<(SongLink, MediaType)>> {
        let query = query_builder::select(Self::table_name())
            .join(
                MediaType::table_name(),
                "type",
                "`media_type`.`name`",
                Join::Inner,
            )
            .fields(SongLinkWithTypeRow::field_names())
            .order_by("song_link.name, `order`", Order::Asc)
            .build();

        crate::db::load::<SongLinkWithTypeRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
    }

    pub fn create(song_id: i32, new_link: NewSongLink, conn: &mut Conn) -> GreaseResult<i32> {
        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;

        let target = check_for_music_file(
            &utf8_percent_encode(&new_link.target, DEFAULT_ENCODE_SET).to_string(),
        )?;
        let query = query_builder::insert(Self::table_name())
            .set("song", &song_id.to_string())
            .set("type", &new_link.type_.to_value().as_sql(false))
            .set("name", &new_link.name.to_value().as_sql(false))
            .set("target", &target.to_value().as_sql(false))
            .build();
        transaction.query(query).map_err(GreaseError::DbError)?;

        let id_query = query_builder::select(Self::table_name())
            .fields(&["id"])
            .order_by("id", Order::Desc)
            .build();

        match transaction.first(id_query).map_err(GreaseError::DbError)? {
            Some(id) => {
                transaction.commit().map_err(GreaseError::DbError)?;
                Ok(id)
            }
            None => Err(GreaseError::ServerError(
                "error inserting new song link".to_owned(),
            )),
        }
    }

    pub fn delete(link_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let link = Self::load(link_id, conn)?;
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("id = {}", link_id))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

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

    pub fn update(link_id: i32, updated_link: SongLinkUpdate, conn: &mut Conn) -> GreaseResult<()> {
        let old_link = Self::load(link_id, conn)?;
        let new_target = utf8_percent_encode(&updated_link.target, DEFAULT_ENCODE_SET).to_string();

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
                    "link '{}' has no associated file. Consider deleting and recreating this link.",
                    old_link.name
                )));
            }
        }

        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", link_id))
            .set("name = '{}'", &updated_link.name)
            .set("target = '{}'", &new_target)
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
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

#[derive(FromRow, FieldNames)]
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
