use crate::error::*;
use db::{Song, SongLink, NewSong, SongUpdate, SongLinkUpdate, NewSongLink, MediaType, schema::StorageType};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use util::check_for_music_file;


impl Song {
    pub fn load<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<Song> {
        use db::schema::song::dsl::*;

        song.filter(id.eq(song_id))
        .first(conn)
        .optional()
        .map_err(GreaseError::DbError)?
        .ok_or(format!("No song with id {}.", song_id))
    }

    pub fn load_with_data<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<SongData> {
        use db::schema::song::dsl::*;
        use db::schema::song_link::{dsl as link_dsl};
        use db::schema::media_type::{dsl as media_type_dsl};

        let song = Song::load(song_id, conn)?;

        let song_links = link_dsl::song_link.filter(link_dsl::song.eq(song_id))
            .order_by(link_dsl::name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)?;
        let mut sorted_links = song_links
            .into_iter()
            .fold(HashMap::new(), |mut map, song_link| {
                let section = map.entry(song_link.type_.clone()).or_insert(Vec::new());
                section.push(song_link);
                map
            });

        let media_types = media_type_dsl::media_type.order_by(media_type_dsl::order.asc())
            .load(conn)
            .map_err(GreaseError::DbError)?;
        let links = media_types
            .into_iter()
            .map(|media_type| {
                let links = sorted_links.remove(&media_type.name).unwrap_or(Vec::new());

                SongLinkSection {
                    name: media_type.name,
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
        use db::schema::song::dsl::*;

        song.order_by(title.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create<C: Connection>(new_song: &NewSong, conn: &mut C) -> GreaseResult<i32> {
        use db::schema::song::dsl::*;

        diesel::insert_into(song)
            .values(new_song)
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update<C: Connection>(
        song_id: i32,
        updated_song: &SongUpdate,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::update(song.filter(id.eq(song_id)))
            .set(updated_song)
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn set_current_status<C: Connection>(
        song_id: i32,
        is_current: bool,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::update(song.filter(id.eq(song_id)))
            .set(current.eq(is_current))
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn delete<C: Connection>(song_id: i32, conn: &mut C) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::delete(song.filter(id.eq(song_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)
    }
}

impl SongLink {
    pub fn load<C: Connection>(link_id: i32, conn: &mut C) -> GreaseResult<SongLink> {
        use db::schema::song_link::dsl::*;

        song_link.filter(id.eq(link_id))
        .first(conn)
        .map_err(GreaseError::DbError)?
        .ok_or(format!("no song link with id {}", link_id))
    }

    pub fn load_all_with_types<C: Connection>(
        conn: &mut C,
    ) -> GreaseResult<Vec<(SongLink, MediaType)>> {
        use db::schema::song_link::dsl::*;
        use db::schema::media_type;

        song_link.inner_join(media_type::table)
            .order_by((name.asc(), media_type::dsl::order.asc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create<C: Connection>(
        song_id: i32,
        new_link: NewSongLink,
        conn: &mut C,
    ) -> GreaseResult<i32> {
        use db::schema::song_link::dsl::*;

        let media_type = MediaType::load(&new_link.type_, conn)?;
        let encoded_target = if media_type.storage == StorageType::Local {
            check_for_music_file(
                &utf8_percent_encode(&new_link.target, DEFAULT_ENCODE_SET).to_string(),
            )?
        } else {
            new_link.target
        };

        conn.transaction(|| {
            diesel::insert_into(song_link)
                .values((
                    song.eq(song_id),
                    type_.eq(new_link.type_),
                    name.eq(new_link.name),
                    target.eq(encoded_target),
                ))
                .execute(conn)?;

            song_link.select(id).order_by(id.desc()).first(conn)
        }).map_err(GreaseError::DbError)
    }

    pub fn delete<C: Connection>(link_id: i32, conn: &mut C) -> GreaseResult<()> {
        use db::schema::song_link::dsl::*;

        let link = SongLink::load(link_id, conn)?;
        let media_type = MediaType::load(&link.type_, conn)?;

        diesel::delete(song_link.filter(id.eq(link_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        if media_type.storage == StorageType::Local {
            let file_name =
                PathBuf::from_str(&format!("../httpsdocs/music/{}", &link.target)).map_err(|_err| {
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
        }

        Ok(())
    }

    pub fn update(
        link_id: i32,
        updated_link: SongLinkUpdate,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::song_link::dsl::*;

        let old_link = SongLink::load(link_id, conn)?;
        let media_type = MediaType::load(&old_link.type_, conn)?;
        let new_target = if &media_type.storage == &StorageType::Local {
            utf8_percent_encode(&updated_link.target, DEFAULT_ENCODE_SET).to_string()
        } else {
            updated_link.target.clone()
        };

        conn.transaction(|| {
            diesel::update(song_link.filter(id.eq(link_id)))
                .set(&updated_link)
                .execute(conn)?;

            if old_link.target != new_target && &media_type.storage == &StorageType::Local {
                let old_path =
                    PathBuf::from_str(&format!("../httpsdocs/music/{}", old_link.target)).map_err(|_err| {
                        GreaseError::ServerError(format!("invalid file name: {}", old_link.target))
                    })?;
                let new_path =
                    PathBuf::from_str(&format!("../httpsdocs/music/{}", new_target)).map_err(|_err| {
                        GreaseError::ServerError(format!("invalid file name: {}", new_target))
                    })?;
                if std::fs::metadata(&old_path).is_ok() {
                    std::fs::rename(&old_path, &new_path).map_err(|err| {
                        GreaseError::ServerError(format!("error renaming link target: {}", err))
                    })?;
                } else {
                    return Err(GreaseError::BadRequest(format!(
                        "Song link '{}' has no associated file. Consider deleting and recreating the link.",
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
    #[serde(flatten)]
    pub song: Song,
    pub links: Vec<SongLinkSection>,
}

#[derive(Serialize)]
pub struct SongLinkSection {
    pub name: String,
    pub links: Vec<SongLink>,
}
