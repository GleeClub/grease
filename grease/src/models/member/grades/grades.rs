use async_graphql::SimpleObject;
use crate::db_conn::DbConn;
use chrono::{Datelike, Duration, Local, NaiveDateTime};
// use crate::models::member::MemberAttendance;

#[derive(SimpleObject)]
pub struct GradeChange {
    /// The reason the grade change was incurred
    pub reason: String,
    /// How much the grade changed
    pub change: f64,
    /// What the final grade was up to this event
    pub partial_score: f64,
}

pub struct EventWithGradeChange {
    pub event: Event,
    pub change: GradeChange,
}

pub struct Grades {
    pub final_grade: f64,
    pub volunteer_gigs_attended: usize,
    pub events_with_changes: Vec<EventWithGradeChange>,
}

