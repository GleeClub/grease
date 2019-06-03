use db::models::*;
use db::schema::files::dsl::*;
use diesel::pg::PgConnection;
use diesel::*;
use std::fs::remove_file;

impl File {
    pub fn load(given_file_id: i32, conn: &PgConnection) -> Result<File, String> {
        files
            .filter(id.eq(given_file_id))
            .first::<File>(conn)
            .optional()
            .expect("error loading file")
            .ok_or(format!("no file exists with the id {}", given_file_id))
    }

    pub fn load_for_path(given_file_path: &str, conn: &PgConnection) -> Result<File, String> {
        files
            .filter(path.eq(given_file_path))
            .first::<File>(conn)
            .optional()
            .expect("error loading file")
            .ok_or(format!("no file exists with the path {}", given_file_path))
    }

    pub fn load_for_song(given_song_id: i32, conn: &PgConnection) -> Vec<File> {
        files
            .filter(song_id.eq(given_song_id))
            .order(name)
            .load::<File>(conn)
            .expect("error loading files")
    }

    pub fn load_for_song_sorted(given_song_id: i32, conn: &PgConnection) -> (Vec<File>, Vec<File>) {
        let mut sheets = Vec::new();
        let mut midis = Vec::new();
        for file in File::load_for_song(given_song_id, conn) {
            if file.is_sheet {
                sheets.push(file);
            } else {
                midis.push(file);
            }
        }

        (sheets, midis)
    }

    // TODO: figure out what to do with actual file uploading / creation
    pub fn create(new_file: NewFile, conn: &PgConnection) -> i32 {
        diesel::insert_into(files)
            .values(&new_file)
            .execute(conn)
            .expect("error adding new file");

        files
            .filter(song_id.eq(new_file.song_id))
            .filter(name.eq(new_file.name))
            .first::<File>(conn)
            .expect("error loading file")
            .id
    }

    pub fn create_multiple(new_files: Vec<NewFile>, conn: &PgConnection) {
        diesel::insert_into(files)
            .values(&new_files)
            .execute(conn)
            .expect("error adding new files");
    }

    pub fn update(given_file_id: i32, updated_file: NewFile, conn: &PgConnection) -> bool {
        diesel::update(files.find(given_file_id))
            .set(&updated_file)
            .get_result::<File>(conn)
            .is_ok()
    }

    pub fn remove(given_file_id: i32, conn: &PgConnection) -> bool {
        if let Ok(file_path) = File::load(given_file_id, conn).map(|f| f.path) {
            diesel::delete(files.filter(id.eq(given_file_id)))
                .execute(conn)
                .expect("error removing file");
            remove_file(file_path).is_ok()
        } else {
            false
        }
    }
}

impl PublicJson for File {}
