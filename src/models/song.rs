use std::fs;
use std::path::PathBuf;

use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::file::MusicFile;

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "pitch", rename_all = "snake_case")]
pub enum Pitch {
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
    BSharp,
    CFlat,
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    ESharp,
    FFlat,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "song_mode", rename_all = "snake_case")]
pub enum SongMode {
    Major,
    Minor,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Song {
    /// The ID of the song
    pub id: i64,
    /// The title of the song
    pub title: String,
    /// Any information related to the song
    /// (minor changes to the music, who wrote it, soloists, etc.)
    pub info: Option<String>,
    /// Whether it is in this semester's repertoire
    pub current: bool,
    /// The key of the song
    pub key: Option<Pitch>,
    /// The starting pitch for the song
    pub starting_pitch: Option<Pitch>,
    /// The mode of the song (Major or Minor)
    pub mode: Option<SongMode>,
}

#[ComplexObject]
impl Song {
    /// The links connected to the song sorted into sections
    pub async fn links(&self, ctx: &Context<'_>) -> Result<Vec<SongLinkSection>> {
        let pool: &PgPool = ctx.data_unchecked();
        let mut all_links = SongLink::for_song(self.id, pool).await?;
        let all_types = MediaType::all(pool).await?;

        Ok(all_types
            .into_iter()
            .map(|t| SongLinkSection {
                name: t.name.clone(),
                links: all_links.drain_filter(|l| &l.r#type == &t.name).collect(),
            })
            .collect())
    }
}

impl Song {
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No song with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, title, info, current, key as \"key: _\",
                 starting_pitch as \"starting_pitch: _\", mode as \"mode: _\"
             FROM song WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, title, info, current, key as \"key: _\",
                 starting_pitch as \"starting_pitch: _\", mode as \"mode: _\"
             FROM song ORDER BY title"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    // TODO: fix query
    pub async fn setlist_for_event(event_id: i64, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT s.id, s.title, s.info, s.current, s.key as \"key: _\",
                 starting_pitch as \"starting_pitch: _\", mode as \"mode: _\"
             FROM song s INNER JOIN gig_song ON s.id = gig_song.song
             WHERE gig_song.event = $1 ORDER BY gig_song.order ASC",
            event_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(new_song: NewSong, pool: &PgPool) -> Result<i64> {
        sqlx::query!(
            "INSERT INTO song (title, info) VALUES ($1, $2)",
            new_song.title,
            new_song.info
        )
        .execute(pool)
        .await?;

        sqlx::query_scalar!("SELECT id FROM song ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, updated_song: SongUpdate, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "UPDATE song SET title = $1, current = $2, info = $3, key = $4, starting_pitch = $5, mode = $6 WHERE id = $7",
            updated_song.title, updated_song.current, updated_song.info, updated_song.key as _, updated_song.starting_pitch as _, updated_song.mode as _, id
        ).execute(pool).await?;

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        // TODO: verify exists
        sqlx::query!("DELETE FROM song WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(SimpleObject)]
pub struct PublicSong {
    pub title: String,
    pub current: bool,
    pub videos: Vec<PublicVideo>,
}

impl PublicSong {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        let mut all_public_videos = sqlx::query!(
            "SELECT name, target, song FROM song_link WHERE type = $1",
            SongLink::PERFORMANCES
        )
        .fetch_all(pool)
        .await?;
        let all_public_songs = sqlx::query!("SELECT id, title, current FROM song ORDER BY title")
            .fetch_all(pool)
            .await?;

        Ok(all_public_songs
            .into_iter()
            .map(|ps| PublicSong {
                title: ps.title,
                current: ps.current,
                videos: all_public_videos
                    .drain_filter(|pv| pv.song == ps.id)
                    .map(|pv| PublicVideo {
                        title: pv.name,
                        url: pv.target,
                    })
                    .collect(),
            })
            .collect())
    }
}

#[derive(SimpleObject)]
pub struct PublicVideo {
    pub title: String,
    pub url: String,
}

#[derive(SimpleObject)]
pub struct SongLinkSection {
    pub name: String,
    pub links: Vec<SongLink>,
}

#[derive(SimpleObject)]
pub struct GigSong {
    pub event: i64,
    pub song: i64,
    pub order: i64,
}

#[derive(SimpleObject)]
pub struct MediaType {
    /// The name of the type of media
    pub name: String,
    /// The order of where this media type appears in a song's link section
    pub order: i64,
    /// The type of storage that this type of media points to
    pub storage: StorageType,
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "storage_type", rename_all = "snake_case")]
pub enum StorageType {
    Local,
    Remote,
}

impl MediaType {
    pub async fn with_name(name: &str, pool: &PgPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No media type named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, \"order\", storage as \"storage: _\"
             FROM media_type WHERE name = $1",
            name
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        // TODO: grep ASC -> remove all instances
        sqlx::query_as!(
            Self,
            "SELECT name, \"order\", storage as \"storage: _\"
             FROM media_type ORDER BY \"order\""
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

#[derive(SimpleObject)]
pub struct SongLink {
    /// The ID of the song link
    pub id: i64,
    /// The ID of the song this link belongs to
    pub song: i64,
    /// The type of this link (e.g. MIDI)
    pub r#type: String,
    /// The name of this link
    pub name: String,
    /// The target this link points to
    pub target: String,
}

impl SongLink {
    pub const PERFORMANCES: &'static str = "Performances";

    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No song link with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE id = $1", id)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn for_song(song_id: i64, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE song = $1", song_id)
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(song_id: i64, new_link: NewSongLink, pool: &PgPool) -> Result<i64> {
        let encoded_target = if let Some(file) = new_link.link_file() {
            file.save()?;
            file.path.to_string_lossy().to_string()
        } else {
            new_link.target
        };

        sqlx::query!(
            "INSERT INTO song_link (song, type, name, target) VALUES ($1, $2, $3, $4)",
            song_id,
            new_link.r#type,
            new_link.name,
            encoded_target
        )
        .execute(pool)
        .await?;

        sqlx::query_scalar!("SELECT id FROM song_link ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, update: SongLinkUpdate, pool: &PgPool) -> Result<()> {
        let song_link = SongLink::with_id(id, pool).await?;

        let media_type = MediaType::with_name(&song_link.r#type, pool).await?;
        let new_target = if media_type.storage == StorageType::Local {
            // TODO: is this correct?
            base64::encode(&update.target)
        } else {
            update.target
        };

        sqlx::query!(
            "UPDATE song_link SET name = $1, target = $2 WHERE id = $3",
            update.name,
            new_target,
            id,
        )
        .execute(pool)
        .await?;

        if song_link.target != new_target && media_type.storage == StorageType::Local {
            let old_path = MusicFile::named(&song_link.target)?;
            let new_path = MusicFile::named(&new_target)?;

            // TODO: verify behavior
            if MusicFile::exists(&old_path)? {
                return Err(format!("Song link {} has no associated file", id).into());
            } else {
                fs::rename(old_path, new_path)?;
            }
        }

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        let song_link = SongLink::with_id(id, pool).await?;
        let media_type = MediaType::with_name(&song_link.r#type, pool).await?;

        sqlx::query!("DELETE FROM song_link WHERE id = $1", id)
            .execute(pool)
            .await?;

        if media_type.storage == StorageType::Local && MusicFile::exists(&song_link.target)? {
            fs::remove_file(MusicFile::named(song_link.target)?)?;
        }

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewSong {
    pub title: String,
    pub info: Option<String>,
}

#[derive(InputObject)]
pub struct SongUpdate {
    pub title: String,
    pub current: bool,
    pub info: Option<String>,
    pub key: Option<Pitch>,
    pub starting_pitch: Option<Pitch>,
    pub mode: Option<SongMode>,
}

#[derive(InputObject)]
pub struct NewSongLink {
    pub r#type: String,
    pub name: String,
    pub target: String,
    pub content: Option<String>,
}

impl NewSongLink {
    pub fn link_file(&self) -> Option<MusicFile> {
        if let Some(data) = &self.content {
            let data = base64::decode(&data).ok()?;
            let path = PathBuf::from(&self.target);

            Some(MusicFile {
                path,
                content: data,
            })
        } else {
            None
        }
    }
}

#[derive(InputObject)]
pub struct SongLinkUpdate {
    pub name: String,
    pub target: String,
}
