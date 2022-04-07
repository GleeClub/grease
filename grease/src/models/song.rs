use async_graphql::{Enum, SimpleObject, ComplexObject, InputObject};
use crate::db_conn::DbConn;

#[derive(Enum)]
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

#[derive(Enum)]
pub enum Mode {
    Major,
    Minor,
}

#[derive(SimpleObject)]
pub struct Song {
    /// The ID of the song
    pub id: isize,
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
    pub mode: Option<Mode>,
}

#[ComplexObject]
impl Song {
    /// The links connected to the song sorted into sections
    pub async fn links(&self, ctx: &Context<'_>) -> Result<Vec<SongLinkSection>> {
        let conn = ctx.data_unchecked::<DbConn>();
        let mut all_links = SongLink::for_song(self.id, conn).await?;
        let all_types = MediaType::all(conn).await?;

        Ok(all_types.into_iter().map(|t| SongLinkSection {
            name: t.name,
            links: all_links.drain_filter(|l| l.r#type == t.name).collect(),
        }))
    }
}

impl Song {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| format!("No song with id {}", id))
    }

    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song WHERE id = ?", id).query_optional(conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song ORDER BY title").query_all(conn).await
    }

    pub async fn setlist_for_event(event_id: isize, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self, "SELECT * FROM song INNER JOIN gig_song ON song.id = gig_song.song
                   WHERE gig_song.event = ? ORDER BY gig_song.order ASC", event_id).query_all(conn).await
    }

    pub async fn create(new_song: NewSong, conn: &DbConn) -> Result<isize> {
        sqlx::query!("INSERT INTO song (title, info) VALUES (?, ?)", new_song.title, new_song.info).query(conn).await?;

        sqlx::query!("SELECT id FROM song ORDER BY id DESC").query_one(conn).await
    }

    pub async fn update(id: isize, updated_song: SongUpdate, conn: &DbConn) -> Result<()> {
        sqlx::query!(
            "UPDATE song SET title = ?, current = ?, info = ?, `key` = ?, starting_pitch = ?, mode = ? WHERE id = ?",
            updated_song.title, updated_song.current, updated_song.info, updated_song.key, updated_song.starting_pitch, updated_song.mode, id
        ).query(conn).await
    }

    pub async fn delete(id: isize, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        sqlx::query!("DELETE FROM song WHERE id = ?", id).query(conn).await
    }
}

#[derive(SimpleObject)]
pub struct PublicSong {
    pub title: String,
    pub current: bool,
    pub videos: Vec<PublicVideo>,
}

impl PublicSong {
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        let mut all_public_videos = sqlx::query!(
            "SELECT name, target, song FROM song_link WHERE `type` = ?",
                SongLink::PERFORMANCES).query_all(conn).await?;
         let all_public_songs = sqlx::query!("SELECT id, title, current FROM song ORDER BY title").query_all(conn).await?;

         Ok(all_public_songs.into_iter().map(|ps| PublicSong {
             title: ps.title,
             current: ps.current,
             videos: all_public_videos.drain_filter(|pv| pv.song == ps.id).map(|pv| PublicVideo {
                 title: pv.name,
                 url: pv.target,
             }).collect()
         }))
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
    pub event: isize,
    pub song: isize,
    pub order: isize,
}

#[derive(SimpleObject)]
pub struct MediaType {
    /// The name of the type of media
    pub name: String,
    /// The order of where this media type appears in a song's link section
    pub order: isize,
    /// The type of storage that this type of media points to
    pub storage: StorageType,
}

#[derive(Enum)]
pub enum StorageType {
    Local,
    Remote,
}

impl MediaType {
    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::with_name_opt(name, conn).await?.ok_or_else(|| format!("No media type named {}", name))
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM media_type WHERE name = ?", name).query_optional(conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        // TODO: grep ASC -> remove all instances
        sqlx::query_as!(Self, "SELECT * FROM media_type ORDER BY `order`").query_all(conn).await
    }
}

#[derive(SimpleObject)]
pub struct SongLink {
    /// The ID of the song link
    pub id: isize,
    /// The ID of the song this link belongs to
    pub song: isize,
    /// The type of this link (e.g. MIDI)
    pub r#type: String,
    /// The name of this link
    pub name: String,
    /// The target this link points to
    pub target: String,
}

impl SongLink {
    pub const PERFORMANCES: &'static str = "Performances";

    // class_getter table_name = "song_link"

    pub async fn with_id_opt(id: isize, pool: &MySqlPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE id = ?", id).query_optional(pool).await.into()
    }

    pub async fn with_id(id: isize, pool: &MySqlPool) -> Result<Self> {
        Self::with_id_opt(id, pool).await.and_then(|res| res.ok_or_else(|| format!(
            "No song link with id {}", id)))
    }

    pub async fn for_song(song_id: isize, pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE song = ?", song_id).query_all(pool).await.into()
    }

    pub async fn all(pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link").query_all(pool).await.into()
    }

    pub async fn create(song_id: isize, new_link: NewSongLink, pool: &MySqlPool) -> Result<isize> {
        let encoded_target = if let Some(content) = new_link.content {
            upload_file(new_link.target, content).await?
        } else {
            new_link.target
        };

        pool.begin(|tx| {
            sqlx::query!(
                "INSERT INTO song_link (song, type, name, target) VALUES (?, ?, ?, ?)",
                song_id, new_link.r#type, new_link.name, encoded_target
            ).query(tx).await?;

            sqlx::query!("SELECT id FROM song_link ORDER BY id DESC").query(tx).await.into()
        })
    }

    pub async fn update(&mut self, update: SongLinkUpdate, pool: &MySqlPool) -> Result<()> {
        let media_type = MediaType::with_name(self.r#type).await?;
        let new_target = if media_type.storage == StorageType::Local {
            encode(update.target)
        } else {
            update.target
        };

        pool.begin(|tx| {
            sqlx::query!(
                "UPDATE song_link SET name = ?, target = ? WHERE id = ?",
                update.name, new_target, self.id,
            ).query(tx).await?;

            if self.target != new_target && media_type.storage = StorageType::Local {
                let old_path = MUSIC_BASE_PATH.append(self.target);
                let new_path = MUSIC_BASE_PATH.append(new_target);

                if file_exists(old_path) {
                    return Err(format!("Song link {} has no associated file", self.name));
                } else {
                    rename_file(old_path, new_path).await?;
                }
            }

            self.name = update.name;
            self.target = new_target;

            tx.commit().await?;

            Ok(())
        })
    }

    pub async fn delete(&self, pool: &MySqlPool) -> Result<()> {
        let media_type = MediaType::with_name(self.r#type).await?;

        pool.begin(|tx| {
            sqlx::query!("DELETE FROM song_link WHERE id = ?", self.id).await?;

            if media_type.storage = StorageType::Local {
                let file_name = MUSIC_BASE_PATH.append(self.target);
                if file_exists(file_name).await? {
                    delete_file(file_name)
                }
            }

            tx.commit().await?;

            Ok(())
        })
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
    pub mode: Option<Mode>,
}

#[derive(InputObject)]
pub struct NewSongLink {
    pub r#type: String,
    pub name: String,
    pub target: String,
    pub content: Option<String>,
}

#[derive(InputObject)]
pub struct SongLinkUpdate {
    pub name: String,
    pub target: String,
}
