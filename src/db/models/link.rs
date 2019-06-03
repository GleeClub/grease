use db::models::*;
use db::schema::links::dsl::*;
use diesel::pg::PgConnection;
use diesel::*;

impl Link {
    pub fn load(given_link_id: i32, conn: &PgConnection) -> Result<Link, String> {
        links
            .filter(id.eq(given_link_id))
            .first::<Link>(conn)
            .optional()
            .expect("error loading link")
            .ok_or(format!("no link exists with the id {}", given_link_id))
    }

    pub fn load_for_song(given_song_id: i32, conn: &PgConnection) -> Vec<Link> {
        links
            .filter(song_id.eq(given_song_id))
            .order(name)
            .load::<Link>(conn)
            .expect("error loading links")
    }

    pub fn load_for_song_sorted(given_song_id: i32, conn: &PgConnection) -> (Vec<Link>, Vec<Link>) {
        let mut all_links = Link::load_for_song(given_song_id, conn);
        let performance_links = all_links.drain_filter(|l| l.is_performance).collect();
        let other_links = all_links;

        (performance_links, other_links)
    }

    // TODO: figure out what to do with actual link uploading / creation
    pub fn create(new_link: &NewLink, conn: &PgConnection) -> i32 {
        diesel::insert_into(links)
            .values(new_link)
            .execute(conn)
            .expect("error adding new link");

        links
            .filter(song_id.eq(&new_link.song_id))
            .filter(link.eq(&new_link.link))
            .first::<Link>(conn)
            .expect("error loading link")
            .id
    }

    pub fn create_multiple(new_links: Vec<NewLink>, conn: &PgConnection) {
        diesel::insert_into(links)
            .values(&new_links)
            .execute(conn)
            .expect("error adding new links");
    }

    pub fn update(given_link_id: i32, updated_link: NewLink, conn: &PgConnection) -> bool {
        diesel::update(links.find(given_link_id))
            .set(&updated_link)
            .get_result::<Link>(conn)
            .is_ok()
    }

    pub fn remove(given_link_id: i32, conn: &PgConnection) {
        diesel::delete(links.filter(id.eq(given_link_id)))
            .execute(conn)
            .expect("error removing link");
    }
}

impl PublicJson for Link {}
