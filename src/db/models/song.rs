use db::models::*;
use db::schema::songs::dsl::*;
use diesel::pg::PgConnection;
use diesel::*;
use crate::db::schema::{StorageType, Key, SongMode};

impl Song {
    pub fn load(given_song_id: i32, conn: &PgConnection) -> Result<Song, String> {
        songs
            .filter(id.eq(given_song_id))
            .first::<Song>(conn)
            .optional()
            .expect("error loading song")
            .ok_or(format!("no song exists with the id {}", given_song_id))
    }

    pub fn load_with_data(given_song_id: i32, conn: &PgConnection) -> Result<SongData, String> {
        let song = Song::load(given_song_id, conn)?;
        let (sheets, midis) = File::load_for_song_sorted(given_song_id, conn);
        let (performance_links, other_links) = Link::load_for_song_sorted(given_song_id, conn);
        Ok(SongData {
            song,
            sheets,
            midis,
            performance_links,
            other_links,
        })
    }

    pub fn load_all(conn: &PgConnection) -> Vec<Song> {
        songs
            .order(name)
            .load::<Song>(conn)
            .expect("error loading songs")
    }

    pub fn load_all_separate_this_semester(conn: &PgConnection) -> (Vec<Song>, Vec<Song>) {
        let mut all_songs = Song::load_all(conn);
        let current_songs = all_songs.drain_filter(|s| s.this_semester).collect();
        let other_songs = all_songs;

        (current_songs, other_songs)
    }

    pub fn create(new_song: &NewSong, conn: &PgConnection) {
        diesel::insert_into(songs)
            .values(new_song)
            .execute(conn)
            .expect("error adding new attendances");
    }

    pub fn update(given_song_id: i32, updated_song: NewSong, conn: &PgConnection) -> bool {
        diesel::update(songs.find(given_song_id))
            .set(&updated_song)
            .get_result::<Song>(conn)
            .is_ok()
    }
}

impl SongLink {
    use db::schema::google_docs::dsl::*;

    pub fn load(doc_name: &str, conn: &MysqlConnection) -> GreaseResult<Vec<Document>> {
        google_docs.filter(name.eq(doc_name)).first(conn).map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Document>> {
        google_docs.load(conn).map_err(GreaseError::DbError)
    }
}
