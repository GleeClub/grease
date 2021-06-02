use crate::{db::NewLinkTarget, error::*};
use db::{
    schema::StorageType, MediaType, NewSong, NewSongLink, Song, SongLink, SongLinkUpdate,
    SongUpdate,
};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

#[derive(Serialize)]
pub struct PublicSong {
    title: String,
    current: bool,
    videos: Vec<PublicVideo>,
}

#[derive(Serialize)]
pub struct PublicVideo {
    title: String,
    url: String,
}

impl Song {
    pub fn load(song_id: i32, conn: &MysqlConnection) -> GreaseResult<Song> {
        use db::schema::song::dsl::*;

        song.filter(id.eq(song_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "No song with id {}.",
                song_id
            )))
    }

    pub fn load_with_data(song_id: i32, conn: &MysqlConnection) -> GreaseResult<SongData> {
        use db::schema::media_type::dsl as media_type_dsl;
        use db::schema::song_link::dsl as link_dsl;

        let given_song = Song::load(song_id, conn)?;

        let song_links = link_dsl::song_link
            .filter(link_dsl::song.eq(song_id))
            .order_by(link_dsl::name.asc())
            .load::<SongLink>(conn)
            .map_err(GreaseError::DbError)?;
        let mut sorted_links = song_links
            .into_iter()
            .fold(HashMap::new(), |mut map, song_link| {
                let section = map.entry(song_link.type_.clone()).or_insert(Vec::new());
                section.push(song_link);
                map
            });

        let media_types = media_type_dsl::media_type
            .order_by(media_type_dsl::order.asc())
            .load::<MediaType>(conn)
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
            Ok(SongData {
                song: given_song,
                links,
            })
        }
    }

    pub fn load_all_public(conn: &MysqlConnection) -> GreaseResult<Vec<PublicSong>> {
        let all_songs = Song::load_all(conn)?;
        let mut all_links = SongLink::load_all_with_types(conn)?;

        let public_songs = all_songs
            .into_iter()
            .map(|song| {
                let videos = all_links
                    .drain_filter(|(link, type_)| {
                        type_.name == SongLink::PERFORMANCES && link.song == song.id
                    })
                    .map(|(link, _)| PublicVideo {
                        title: link.name,
                        url: link.target,
                    })
                    .collect();

                PublicSong {
                    title: song.title,
                    current: song.current,
                    videos,
                }
            })
            .collect();

        Ok(public_songs)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Song>> {
        use db::schema::song::dsl::*;

        song.order_by(title.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(new_song: &NewSong, conn: &MysqlConnection) -> GreaseResult<i32> {
        use db::schema::song::dsl::*;

        conn.transaction(|| {
            diesel::insert_into(song).values(new_song).execute(conn)?;

            song.select(id).order_by(id.desc()).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn update(
        song_id: i32,
        updated_song: &SongUpdate,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::update(song.filter(id.eq(song_id)))
            .set(updated_song)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn set_current_status(
        song_id: i32,
        is_current: bool,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::update(song.filter(id.eq(song_id)))
            .set(current.eq(is_current))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(song_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::song::dsl::*;

        diesel::delete(song.filter(id.eq(song_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl SongLink {
    pub const PERFORMANCES: &'static str = "Performances";

    pub fn load(link_id: i32, conn: &MysqlConnection) -> GreaseResult<SongLink> {
        use db::schema::song_link::dsl::*;

        song_link
            .filter(id.eq(link_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "no song link with id {}.",
                link_id
            )))
    }

    pub fn load_all_with_types(conn: &MysqlConnection) -> GreaseResult<Vec<(SongLink, MediaType)>> {
        use db::schema::media_type;
        use db::schema::song_link::dsl::*;

        song_link
            .inner_join(media_type::table)
            .order_by((name.asc(), media_type::dsl::order.asc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(
        song_id: i32,
        new_link: NewSongLink,
        conn: &MysqlConnection,
    ) -> GreaseResult<i32> {
        use db::schema::song_link::dsl::*;

        let encoded_target = match &new_link.target {
            NewLinkTarget::Url(url) => url.clone(),
            NewLinkTarget::File(file) => {
                file.upload()?;
                utf8_percent_encode(&file.path, DEFAULT_ENCODE_SET).to_string()
            }
        };

        conn.transaction(move || {
            diesel::insert_into(song_link)
                .values((
                    song.eq(song_id),
                    type_.eq(new_link.type_),
                    name.eq(new_link.name),
                    target.eq(encoded_target),
                ))
                .execute(conn)?;

            song_link.select(id).order_by(id.desc()).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn delete(link_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::song_link::dsl::*;

        let link = SongLink::load(link_id, conn)?;
        let media_type = MediaType::load(&link.type_, conn)?;

        diesel::delete(song_link.filter(id.eq(link_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        if media_type.storage == StorageType::Local {
            let file_name = PathBuf::from_str(&format!("../httpdocs/music/{}", &link.target))
                .map_err(|_err| {
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
        conn: &MysqlConnection,
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
                    PathBuf::from_str(&format!("../httpdocs/music/{}", old_link.target)).map_err(|_err| {
                        GreaseError::ServerError(format!("invalid file name: {}", old_link.target))
                    })?;
                let new_path =
                    PathBuf::from_str(&format!("../httpdocs/music/{}", new_target)).map_err(|_err| {
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
