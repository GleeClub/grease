use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;

/// A musical note
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

/// Whether a song is in major or minor
#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "song_mode", rename_all = "snake_case")]
pub enum SongMode {
    /// The song is in a major key
    Major,
    /// The song is in a minor key
    Minor,
}

/// A song that the Glee Club performs
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Song {
    /// The ID of the song
    pub id: i64,
    /// The title of the song
    pub title: String,
    /// Any information related to the song
    /// (minor changes to the music, who wrote it, soloists, etc.)
    pub info: String,
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
    /// The sorted sections of links belonging to the song
    pub async fn link_sections(&self, ctx: &Context<'_>) -> Result<Vec<SongLinkSection>> {
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
             FROM songs WHERE id = $1",
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
             FROM songs ORDER BY title"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn setlist_for_event(event_id: i64, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT s.id, s.title, s.info, s.current, s.key as \"key: _\",
                 s.starting_pitch as \"starting_pitch: _\", s.mode as \"mode: _\"
             FROM songs s INNER JOIN gig_songs ON s.id = gig_songs.song
             WHERE gig_songs.event = $1 ORDER BY gig_songs.order",
            event_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(new_song: NewSong, pool: &PgPool) -> Result<i64> {
        sqlx::query!(
            "INSERT INTO songs (title, info) VALUES ($1, $2)",
            new_song.title,
            new_song.info
        )
        .execute(pool)
        .await?;

        sqlx::query_scalar!("SELECT id FROM songs ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, updated_song: SongUpdate, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "UPDATE songs SET title = $1, current = $2, info = $3, key = $4, starting_pitch = $5, mode = $6 WHERE id = $7",
            updated_song.title, updated_song.current, updated_song.info, updated_song.key as _, updated_song.starting_pitch as _, updated_song.mode as _, id
        ).execute(pool).await?;

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        // verify exists
        Song::with_id(id, pool).await?;

        sqlx::query!("DELETE FROM songs WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// A song that is visible on the external site
#[derive(SimpleObject)]
pub struct PublicSong {
    /// The title of the song
    pub title: String,
    /// Whether the song is in the current club repertoire
    pub current: bool,
    /// Links to YouTube performances of this song by the Glee Club
    pub videos: Vec<PublicVideo>,
}

impl PublicSong {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        let mut all_public_videos = sqlx::query!(
            "SELECT name, url, song FROM song_links WHERE type = $1",
            SongLink::PERFORMANCES
        )
        .fetch_all(pool)
        .await?;
        let all_public_songs = sqlx::query!("SELECT id, title, current FROM songs ORDER BY title")
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
                        url: pv.url.unwrap(),
                    })
                    .collect(),
            })
            .collect())
    }
}

/// A YouTube performance of a song by the Glee Club
#[derive(SimpleObject)]
pub struct PublicVideo {
    /// The name of the song
    pub title: String,
    /// A link to the performance on YouTube
    pub url: String,
}

/// A group of links to resources for a song
#[derive(SimpleObject)]
pub struct SongLinkSection {
    /// The name of the link group
    pub name: String,
    /// The links in this group
    pub links: Vec<SongLink>,
}

/// A type of media belonging to a song
#[derive(SimpleObject)]
pub struct MediaType {
    /// The name of the type of media
    pub name: String,
    /// The order of where this media type appears in a song's link section
    pub order: i64,
    /// The type of storage that this type of media points to
    pub storage: StorageType,
}

/// Whether a media item is a link or a local file
#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "storage_type", rename_all = "snake_case")]
pub enum StorageType {
    /// The item is stored locally
    Local,
    /// The item is a link to an external resource
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
             FROM media_types WHERE name = $1",
            name
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, \"order\", storage as \"storage: _\"
             FROM media_types ORDER BY \"order\""
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

/// A link to some media under a song
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct SongLink {
    /// The ID of the song link
    pub id: i64,
    /// The ID of the song this link belongs to
    pub song: i64,
    /// The type of this link (e.g. MIDI)
    pub r#type: String,
    /// The name of this link
    pub name: String,

    #[graphql(skip)]
    pub url: Option<String>,
    #[graphql(skip)]
    pub file: Option<String>,
}

#[ComplexObject]
impl SongLink {
    /// The URL this link points to
    pub async fn url(&self) -> Result<String> {
        if let Some(url) = &self.url {
            Ok(url.clone())
        } else if let Some(file) = &self.file {
            Ok(format!("https://api.glubhub.org/files/{}", file))
        } else {
            Err("Song link is malformed and has no URL".into())
        }
    }
}

impl SongLink {
    pub const PERFORMANCES: &'static str = "Performances";

    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No song link with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_links WHERE id = $1", id)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn for_song(song_id: i64, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM song_links WHERE song = $1 ORDER BY type",
            song_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(song_id: i64, new_link: NewSongLink, pool: &PgPool) -> Result<i64> {
        MediaType::with_name(&new_link.r#type, pool).await?;

        if let Some(content) = new_link.content {
            let data = base64::decode(&content)
                .map_err(|err| format!("Failed to decode file content: {err}"))?;

            sqlx::query!(
                "INSERT INTO song_files (name, data) VALUES ($1, $2)",
                new_link.url,
                data
            )
            .execute(pool)
            .await?;
            sqlx::query!(
                "INSERT INTO song_links (song, type, name, url, file)
                 VALUES ($1, $2, $3, $4, $5)",
                song_id,
                new_link.r#type,
                new_link.name,
                Option::<String>::None,
                new_link.url,
            )
            .execute(pool)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO song_links (song, type, name, url, file)
                 VALUES ($1, $2, $3, $4, $5)",
                song_id,
                new_link.r#type,
                new_link.name,
                new_link.url,
                Option::<String>::None,
            )
            .execute(pool)
            .await?;
        }

        sqlx::query_scalar!("SELECT id FROM song_links ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, update: SongLinkUpdate, pool: &PgPool) -> Result<()> {
        let song_link = SongLink::with_id(id, pool).await?;

        if song_link.url.is_some() {
            sqlx::query!(
                "UPDATE song_links SET name = $1, url = $2 WHERE id = $3",
                update.name,
                update.url,
                id
            )
            .execute(pool)
            .await?;
        } else {
            let old_file_name = song_link
                .file
                .ok_or_else(|| "Can't update file name because old name is missing")?;

            sqlx::query!(
                "UPDATE song_links SET name = $1 WHERE id = $2",
                update.name,
                id
            )
            .execute(pool)
            .await?;
            sqlx::query!(
                "UPDATE song_files SET name = $1 WHERE name = $2",
                update.url,
                old_file_name,
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        let song_link = SongLink::with_id(id, pool).await?;

        if let Some(file) = song_link.file {
            sqlx::query!("DELETE FROM song_files WHERE name = $1", file)
                .execute(pool)
                .await?;
        }

        sqlx::query!("DELETE FROM song_links WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// A new song for the club to perform
#[derive(InputObject)]
pub struct NewSong {
    /// The name of the new song
    pub title: String,
    /// A description of the song
    pub info: Option<String>,
}

/// An update to an existing song
#[derive(InputObject)]
pub struct SongUpdate {
    /// The new name for the song
    pub title: String,
    /// Whether the song is in the club's current repertoire
    pub current: bool,
    /// A description of the song
    pub info: String,
    /// The key of the song
    pub key: Option<Pitch>,
    /// The pitch the song starts on
    pub starting_pitch: Option<Pitch>,
    /// Whether the song is in major or minor
    pub mode: Option<SongMode>,
}

/// A new link to media under a song
#[derive(InputObject)]
pub struct NewSongLink {
    /// The type of the media
    pub r#type: String,
    /// The name of the resource
    pub name: String,
    /// A link to the media
    pub url: String,
    /// The content of the link
    pub content: Option<String>,
}

/// An update to a song link
#[derive(InputObject)]
pub struct SongLinkUpdate {
    /// The new name of the link
    pub name: String,
    /// The new URL for the link
    pub url: String,
}
