use async_graphql::{Enum, ComplexObject};
use sqlx::MySqlConnection;

#[derive(Enum)]
pub enum Pitch {
    AFlat,
    A,
    ASharp,
    BFlat,
    B
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
#[graphql(complex)]
pub struct Song {
    /// The ID of the song
    pub id: i32,
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

impl Song {
    pub async fn with_id_opt(id: i32, conn: &mut MySqlConnection) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song WHERE id = ?", id).query_optional(conn).await.into()
    }

    pub async fn with_id(id: i32, conn: &mut MySqlConnection) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| anyhow::anyhow!("No song with id {}", id))
    }

    pub async fn all(conn: &mut MySqlConnection) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song ORDER BY title").query_all(conn).await.into()
    }

    def self.setlist_for_event(event_id)
      CONN.query_all "SELECT * FROM #{@@table_name} \
        INNER JOIN #{GigSong.table_name} ON #{@@table_name}.id = #{GigSong.table_name}.song
        WHERE #{GigSong.table_name}.event = ?
        ORDER BY #{GigSong.table_name}.order ASC", event_id, as: Song
    end

    def self.all_public
      all_links = SongLink.all.select { |link| link.type == SongLink::PERFORMANCES }

      Song.all.map do |song|
        videos = all_links
          .select { |link| link.song == song.id }
          .map { |link| PublicVideo.new link.name, link.target }

        PublicSong.new song.title, song.current, videos
      end
    end

    def self.create(form)
      CONN.exec "INSERT INTO #{@@table_name} (title, info) \
        VALUES (?, ?)", form.title, form.info

      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def update(form)
      CONN.exec "SET #{@@table_name} \
        title = ?, current = ?, info = ?, key = ?, starting_pitch = ?, mode = ?
        WHERE id = ?", form.title, form.current, form.info, form.key.try &.to_rs,
        form.starting_pitch.try &.to_rs, form.mode.try &.to_rs, @id
    end

    def delete
      CONN.exec "DELETE #{@@table_name} WHERE id = ?", @id
    end


    @[GraphQL::Field(description: "The links connected to the song sorted into sections")]
    def links : Array(Models::SongLinkSection)
      all_links = SongLink.for_song @id
      all_types = MediaType.all

      all_types.map do |type|
        SongLinkSection.new type.name, all_links.select { |l| l.type == type.name }
      end
    end
  end

  @[GraphQL::Object]
  class SongLinkSection
    include GraphQL::ObjectType

    def initialize(@name : String, @links : Array(SongLink))
    end

    @[GraphQL::Field]
    def name : String
      @name
    end

    @[GraphQL::Field]
    def links : Array(Models::SongLink)
      @links
    end
  end

  class GigSong
    class_getter table_name = "gig_song"

    DB.mapping({
      event: Int32,
      song:  Int32,
      order: Int32,
    })
  end

  @[GraphQL::Object]
  class MediaType
    include GraphQL::ObjectType

    class_getter table_name = "media_type"

    @[GraphQL::Enum]
    enum StorageType
      LOCAL
      REMOTE

      def self.mapping
        {
          "LOCAL"  => LOCAL,
          "REMOTE" => REMOTE,
        }
      end

      def to_rs
        StorageType.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        storage_type = val.as?(String).try { |v| StorageType.mapping[v.upcase]? }
        storage_type || raise "Invalid storage type returned from database: #{val}"
      end

      def self.parse(val)
        StorageType.mapping[val]? || raise "Invalid storage type variant provided: #{val}"
      end
    end

    DB.mapping({
      name:    String,
      order:   Int32,
      storage: {type: StorageType, converter: StorageType},
    })

    def self.with_name(name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE name = ?", name, as: MediaType
    end

    def self.with_name!(name)
      (with_name name) || raise "No media type named #{name}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY order ASC", as: MediaType
    end

    @[GraphQL::Field(description: "The name of the type of media")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "The order of where this media type appears in a song's link section")]
    def order : Int32
      @order
    end

    @[GraphQL::Field(description: "The type of storage that this type of media points to")]
    def storage : Models::MediaType::StorageType
      @storage
    end
  end

#[derive(SimpleObject)]
pub struct SongLink {
    /// The ID of the song link
    pub id: i32,
    /// The ID of the song this link belongs to
    pub song: i32,
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

    pub async fn with_id_opt(id: i32, pool: &MySqlPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE id = ?", id).query_optional(pool).await.into()
    }

    pub async fn with_id(id: i32, pool: &MySqlPool) -> Result<Self> {
        Self::with_id_opt(id, pool).await.and_then(|res| res.ok_or_else(|| format!(
            "No song link with id {}", id)))
    }

    pub async fn for_song(song_id: i32, pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link WHERE song = ?", song_id).query_all(pool).await.into()
    }

    pub async fn all(pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM song_link").query_all(pool).await.into()
    }

    pub async fn create(song_id: i32, new_link: NewSongLink, pool: &MySqlPool) -> Result<i32> {
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

#[derive(SimpleObject)]
pub struct PublicSong {
    pub title: String,
    pub current: bool,
    pub videos: Vec<PublicVideo>,
}

#[derive(SimpleObject)]
pub struct PublicVideo {
    pub title: String,
    pub url: String,
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
