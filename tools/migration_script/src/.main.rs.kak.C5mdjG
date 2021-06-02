#![feature(drain_filter)]

extern crate bcrypt;
extern crate chrono;
extern crate mysql;
extern crate structopt;
extern crate url;

mod error;
mod migrate;
mod new_schema;
mod old_schema;

use error::{MigrateError, MigrateResult};
use migrate::{Load, Migrate};
use mysql::{Conn, Pool};
use new_schema::*;
use old_schema::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Passwords {
    #[structopt(short = "o", long = "old")]
    pub mensgleeclub: String,

    #[structopt(short = "n", long = "new")]
    pub glubhub: String,
}

fn main() {
    let passwords = Passwords::from_args();

    if let Err(migrate_error) = run_migration(&passwords) {
        println!("{}", migrate_error.stringify());
    }
}

pub fn run_migration(passwords: &Passwords) -> MigrateResult<()> {
    let old_url = format!(
        "mysql://chris:{}@127.0.0.1/mensgleeclub",
        &passwords.mensgleeclub
    );
    let new_url = format!("mysql://master:{}@127.0.0.1/glubhub", &passwords.glubhub);

    println!("Connecting to old database...");
    let old_db = Pool::new(&old_url).map_err(MigrateError::MySqlError)?;

    println!("Setting up new database...");
    {
        let temp_new_url = format!("mysql://master:{}@127.0.0.1/", &passwords.glubhub);
        let mut new_db_conn = Conn::new(&temp_new_url).map_err(MigrateError::MySqlError)?;

        println!("(Dropping database if it exists...)");
        new_db_conn
            .query("DROP DATABASE IF EXISTS glubhub;")
            .map_err(MigrateError::MySqlError)?;

        println!("(Creating the database anew...)");
        new_db_conn
            .query("CREATE DATABASE glubhub;")
            .map_err(MigrateError::MySqlError)?;
        new_db_conn
            .query("USE glubhub;")
            .map_err(MigrateError::MySqlError)?;

        println!("(Supplying the schema...)");
        new_db_conn
            .query(include_str!(
                "../../../migrations/2018-08-22-214705_create_tables/up.sql"
            ))
            .map_err(MigrateError::MySqlError)?;
    }

    println!("Connecting to new database...");

    let new_db = Pool::new(&new_url).map_err(MigrateError::MySqlError)?;

    println!("Beginning migrations...");

    println!("Loading old choirs...");
    let old_choirs = OldChoir::load(&old_db)?;

    println!("Migrating variables...");
    let (old_variables, _new_variables) = NewVariable::migrate(&old_db, &new_db, &old_choirs)?;

    println!("Migrating members...");
    let (_old_members, new_members) = NewMember::migrate(&old_db, &new_db, &())?;

    println!("Migrating semesters...");
    let (old_semesters, _new_semesters) = NewSemester::migrate(&old_db, &new_db, &old_variables)?;

    println!("Migrating roles...");
    let (old_roles, _new_roles) = NewRole::migrate(&old_db, &new_db, &())?;

    println!("Migrating member roles...");
    let (_old_member_roles, _new_member_roles) =
        NewMemberRole::migrate(&old_db, &new_db, &old_roles)?;

    println!("Migrating section types...");
    let (old_section_types, _new_section_types) = NewSectionType::migrate(&old_db, &new_db, &())?;

    println!("Migrating event types...");
    let (old_event_types, _new_event_types) = NewEventType::migrate(&old_db, &new_db, &())?;

    println!("Migrating events...");
    let (old_events, _new_events) = NewEvent::migrate(
        &old_db,
        &new_db,
        &(old_section_types.clone(), old_event_types.clone()),
    )?;

    println!("Migrating uniforms...");
    let (old_uniforms, new_uniforms) = NewUniform::migrate(&old_db, &new_db, &())?;

    println!("Migrating gigs...");
    let (_old_gigs, _new_gigs) = NewGig::migrate(&old_db, &new_db, &(old_uniforms, new_uniforms))?;

    println!("Migrating gig requests...");
    let (_old_gig_requests, _new_gig_requests) = NewGigRequest::migrate(&old_db, &new_db, &())?;

    println!("Migrating absence requests...");
    let (_old_absence_requests, _new_absence_requests) =
        NewAbsenceRequest::migrate(&old_db, &new_db, &())?;

    println!("Migrating active semesters...");
    let (old_active_semesters, _new_active_semesters) =
        NewActiveSemester::migrate(&old_db, &new_db, &old_section_types)?;

    println!("Migrating announcements...");
    let (_old_announcements, _new_announcements) =
        NewAnnouncement::migrate(&old_db, &new_db, &old_semesters)?;

    println!("Migrating attendance...");
    let (_old_attendance, _new_attendance) =
        NewAttendance::migrate(&old_db, &new_db, &(old_active_semesters, old_events))?;

    println!("Migrating carpools...");
    let (_old_carpools, _new_carpools) = NewCarpool::migrate(&old_db, &new_db, &())?;

    println!("Migrating fees...");
    let (_old_fees, _new_fees) = NewFee::migrate(&old_db, &new_db, &())?;

    println!("Migrating google docs...");
    let (_old_google_docs, _new_google_docs) = NewGoogleDocs::migrate(&old_db, &new_db, &())?;

    println!("Migrating songs...");
    let (_old_songs, _new_songs) = NewSong::migrate(&old_db, &new_db, &())?;

    println!("Migrating gig songs...");
    let (_old_gig_songs, _new_gig_songs) = NewGigSong::migrate(&old_db, &new_db, &())?;

    println!("Migrating media types...");
    let (old_media_types, _new_media_types) = NewMediaType::migrate(&old_db, &new_db, &())?;

    println!("Migrating song links...");
    let (_old_song_links, _new_song_links) =
        NewSongLink::migrate(&old_db, &new_db, &old_media_types)?;

    println!("Migrating minutes...");
    let (_old_minutes, _new_minutes) = NewMinutes::migrate(&old_db, &new_db, &())?;

    println!("Migrating permissions...");
    let (_old_permissions, _new_permissions) = NewPermission::migrate(&old_db, &new_db, &())?;

    println!("Migrating rides ins...");
    let (_old_rides_ins, _new_rides_ins) = NewRidesIn::migrate(&old_db, &new_db, &())?;

    println!("Migrating role permissions...");
    let (_old_role_permissions, _new_role_permissions) =
        NewRolePermission::migrate(&old_db, &new_db, &(old_roles, old_event_types))?;

    println!("Migrating todos...");
    let (_old_todos, _new_todos) = NewTodo::migrate(&old_db, &new_db, &new_members)?;

    println!("Migrating transaction types...");
    let (old_transaction_types, _new_transaction_types) =
        NewTransactionType::migrate(&old_db, &new_db, &())?;

    println!("Migrating transactions...");
    let (_old_transactions, _new_transactions) =
        NewTransaction::migrate(&old_db, &new_db, &old_transaction_types)?;

    println!("Finished migration!");

    Ok(())
}
