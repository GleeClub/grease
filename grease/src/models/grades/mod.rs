use async_graphql::{Result, SimpleObject};
use time::{Duration, OffsetDateTime};

use crate::db_conn::DbConn;
use crate::models::event::Event;
use crate::models::grades::context::{AttendanceContext, GradesContext};
use crate::models::grades::week::{EventWithAttendance, WeekOfAttendances};

pub mod context;
pub mod week;

#[derive(SimpleObject)]
pub struct Grades {
    /// The overall grade for the semester
    pub grade: f64,
    /// The events of the semester, with the grade changes for those events
    pub events_with_changes: Vec<EventWithGradeChange>,
    /// The number of volunteer gigs attended over the semester
    pub volunteer_gigs_attended: usize,
}

#[derive(SimpleObject)]
pub struct EventWithGradeChange {
    /// The event a grade was received for
    pub event: Event,
    /// What grade change occurred, for what reason
    pub change: GradeChange,
}

#[derive(SimpleObject)]
pub struct GradeChange {
    /// The reason the grade change was incurred
    pub reason: String,
    /// How much the grade changed
    pub change: f64,
    /// What the final grade was up to this event
    pub partial_score: f64,
}

impl Grades {
    pub async fn for_member(email: &str, semester: &str, conn: &DbConn<'_>) -> Result<Grades> {
        let context = GradesContext::for_member_during_semester(email, semester, conn).await?;
        let mut grades = Grades {
            grade: 100.0,
            volunteer_gigs_attended: 0,
            events_with_changes: vec![],
        };

        for week in context.weeks_of_attendance(email) {
            for event in &week {
                let change = Self::calculate_grade_change(event, &week, grades.grade);
                grades.grade = change.partial_score;
                grades.events_with_changes.push(change);

                if week.attended_volunteer_gig().is_some() {
                    grades.volunteer_gigs_attended += 1;
                }
            }
        }

        Ok(grades)
    }

    fn calculate_grade_change(
        event: &EventWithAttendance<'_>,
        context: &WeekOfAttendances<'_>,
        grade: f32,
    ) -> GradeChange {
        let event = &member_attendance.event.event;
        let is_bonus_event = Self::is_bonus_event(&member_attendance, &context);

        let now = OffsetDateTime::now_local().context("Failed to get current time")?;
        let (change, reason) = if event.call_time > now {
            Self::event_hasnt_happened_yet()
        } else if member_attendance.did_attend() {
            if context.missed_rehearsal && event.is_gig() {
                Self::missed_rehearsal(&event)
            } else if member_attendance.minutes_late() > 0 && event.type_ != Event::OMBUDS {
                Self::late_for_event(&member_attendance, grade, is_bonus_event)
            } else if is_bonus_event {
                Self::attended_bonus_event(&event, grade)
            } else {
                Self::attended_normal_event()
            }
        } else if member_attendance.should_attend() {
            Self::should_have_attended(&member_attendance, &context)
        } else {
            Self::didnt_need_to_attend()
        };

        GradeChange {
            reason,
            change,
            partial_score: (grade + change).max(0.0).min(100.0),
        }
    }

    fn attended_normal_event() -> (f32, String) {
        (
            0.0,
            "No point change for attending required event".to_owned(),
        )
    }

    fn didnt_need_to_attend() -> (f32, String) {
        (0.0, "Did not attend and not expected to".to_owned())
    }

    fn event_hasnt_happened_yet() -> (f32, String) {
        (0.0, "Event hasn't happened yet".to_owned())
    }

    fn late_for_event(
        event: EventWithAttendance<'_>,
        grade: f32,
        bonus_event: bool,
    ) -> (f32, String) {
        let points_lost_for_lateness =
            Self::points_lost_for_lateness(event, member_attendance.minutes_late());

        if bonus_event {
            if grade + event.points as f32 - points_lost_for_lateness > 100.0 {
                (
                    100.0 - grade,
                    format!(
                        "Event would grant {}-point bonus, \
                         but {:.2} points deducted for lateness (capped at 100%)",
                        event.points, points_lost_for_lateness
                    ),
                )
            } else {
                (
                    event.points as f32 - points_lost_for_lateness,
                    format!(
                        "Event would grant {}-point bonus, \
                         but {:.2} points deducted for lateness",
                        event.points, points_lost_for_lateness
                    ),
                )
            }
        } else if event.should_attend() {
            (
                -points_lost_for_lateness,
                format!(
                    "{:.2} points deducted for lateness to required event",
                    points_lost_for_lateness
                ),
            )
        } else {
            (
                0.0,
                "No point change for attending required event".to_owned(),
            )
        }
    }

    fn points_lost_for_lateness(event: &Event, minutes_late: i64) -> f32 {
        // Lose points equal to the percentage of the event missed, if they should have attended
        let event_duration = if let Some(release_time) = event.release_time {
            if release_time <= event.call_time {
                60.0
            } else {
                (release_time - event.call_time).num_minutes() as f32
            }
        } else {
            60.0
        };

        (minutes_late as f32 / event_duration) * (event.points as f32)
    }

    fn missed_rehearsal(event: &Event) -> (f32, String) {
        // If you haven't been to rehearsal this week, you can't get points or gig credit
        if event.type_ == Event::VOLUNTEER_GIG {
            (
                0.0,
                format!(
                    "{}-point bonus denied because this week's rehearsal was missed",
                    event.points
                ),
            )
        } else {
            (
                -(event.points as f32),
                "Full deduction for unexcused absence from this week's rehearsal".to_owned(),
            )
        }
    }

    fn attended_bonus_event(event: &Event, grade: f32) -> (f32, String) {
        // Get back points for volunteer gigs and and extra sectionals and ombuds events
        if grade + event.points as f32 > 100.0 {
            let point_change = 100.0 - grade;
            (
                point_change,
                format!(
                    "Event grants {:}-point bonus, but grade is capped at 100%",
                    event.points
                ),
            )
        } else {
            (
                event.points as f32,
                "Full bonus awarded for attending volunteer or extra event".to_owned(),
            )
        }
    }

    fn should_have_attended(
        member_attendance: &MemberAttendance,
        context: &WeeklyAttendanceContext,
    ) -> (f32, String) {
        let event = &member_attendance.event.event;

        // Lose the full point value if did not attend
        if event.type_ == Event::OMBUDS {
            (
                0.0,
                "You do not lose points for missing an ombuds event".to_owned(),
            )
        } else if event.type_ == Event::SECTIONAL && context.attended_sectionals {
            (
                0.0,
                "No deduction because you attended a different sectional this week".to_owned(),
            )
        } else if event.type_ == Event::SECTIONAL
            && context
                .missed_sectional
                .map(|call_time| call_time < event.call_time)
                .unwrap_or(false)
        {
            (
                0.0,
                "No deduction because you already lost points for one sectional this week"
                    .to_owned(),
            )
        } else if event.type_ == Event::SECTIONAL
            && context
                .last_sectional
                .as_ref()
                .map(|last_call_time| {
                    let now = Local::now().naive_utc();
                    last_call_time > &event.call_time && last_call_time > &now
                })
                .unwrap_or(false)
        {
            (
                0.0,
                "No deduction because not all sectionals occurred yet".to_owned(),
            )
        } else if member_attendance.approved_absence() {
            (
                0.0,
                "No deduction because an absence request was submitted and approved".to_owned(),
            )
        } else {
            (
                -(event.points as f32),
                "Full deduction for unexcused absence from event".to_owned(),
            )
        }
    }
}
