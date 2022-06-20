use std::sync::Arc;

use async_graphql::{Result, SimpleObject};
use sqlx::MySqlPool;
use time::OffsetDateTime;

use crate::models::event::{Event, EventType};
use crate::models::grades::context::GradesContext;
use crate::models::grades::week::{EventWithAttendance, WeekOfAttendances};

pub mod context;
pub mod week;

// TODO: refactor this as much as is reasonable

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
    pub event: Arc<Event>,
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
    pub async fn for_member(email: &str, semester: &str, pool: &MySqlPool) -> Result<Grades> {
        let now = crate::util::now()?;
        let context =
            GradesContext::for_members_during_semester(&vec![email], semester, pool).await?;
        let mut grades = Grades {
            grade: 100.0,
            volunteer_gigs_attended: 0,
            events_with_changes: vec![],
        };

        for week in context.weeks_of_attendance(email) {
            for event in week.events.iter() {
                let change = Self::calculate_grade_change(event, &week, grades.grade, &now);
                grades.grade = change.partial_score;
                grades.events_with_changes.push(EventWithGradeChange {
                    event: event.event.clone(),
                    change,
                });

                if week.attended_volunteer_gig(event) {
                    grades.volunteer_gigs_attended += 1;
                }
            }
        }

        Ok(grades)
    }

    fn calculate_grade_change(
        event: &EventWithAttendance<'_>,
        week: &WeekOfAttendances<'_>,
        grade: f64,
        now: &OffsetDateTime,
    ) -> GradeChange {
        let is_bonus_event = week.is_bonus_event(event);

        let (change, reason) = if &event.event.call_time.0 > now {
            Self::event_hasnt_happened_yet()
        } else if event.did_attend() {
            if week.missed_event_of_type(EventType::REHEARSAL).is_some() && event.event.is_gig() {
                Self::missed_rehearsal(&event.event)
            } else if event.minutes_late() > 0 && &event.event.r#type != EventType::OMBUDS {
                Self::late_for_event(event, grade, is_bonus_event)
            } else if is_bonus_event {
                Self::attended_bonus_event(&event.event, grade)
            } else {
                Self::attended_normal_event()
            }
        } else if event.should_attend() {
            Self::should_have_attended(event, week, now)
        } else {
            Self::didnt_need_to_attend()
        };

        GradeChange {
            reason,
            change,
            partial_score: (grade + change).max(0.0).min(100.0),
        }
    }

    fn attended_normal_event() -> (f64, String) {
        (
            0.0,
            "No point change for attending required event".to_owned(),
        )
    }

    fn didnt_need_to_attend() -> (f64, String) {
        (0.0, "Did not attend and not expected to".to_owned())
    }

    fn event_hasnt_happened_yet() -> (f64, String) {
        (0.0, "Event hasn't happened yet".to_owned())
    }

    fn late_for_event(
        event: &EventWithAttendance<'_>,
        grade: f64,
        bonus_event: bool,
    ) -> (f64, String) {
        let points_lost_for_lateness =
            Self::points_lost_for_lateness(&event.event, event.minutes_late());

        if bonus_event {
            if grade + event.event.points as f64 - points_lost_for_lateness > 100.0 {
                (
                    100.0 - grade,
                    format!(
                        "Event would grant {}-point bonus, \
                         but {:.2} points deducted for lateness (capped at 100%)",
                        event.event.points, points_lost_for_lateness
                    ),
                )
            } else {
                (
                    event.event.points as f64 - points_lost_for_lateness,
                    format!(
                        "Event would grant {}-point bonus, \
                         but {:.2} points deducted for lateness",
                        event.event.points, points_lost_for_lateness
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

    fn points_lost_for_lateness(event: &Event, minutes_late: i32) -> f64 {
        // Lose points equal to the percentage of the event missed, if they should have attended
        let event_duration = if let Some(release_time) = &event.release_time {
            if &release_time.0 <= &event.call_time.0 {
                60.0
            } else {
                (release_time.0 - event.call_time.0).whole_minutes() as f64
            }
        } else {
            60.0
        };

        (minutes_late as f64 / event_duration) * (event.points as f64)
    }

    fn missed_rehearsal(event: &Event) -> (f64, String) {
        // If you haven't been to rehearsal this week, you can't get points or gig credit
        if event.r#type == EventType::VOLUNTEER_GIG {
            (
                0.0,
                format!(
                    "{}-point bonus denied because this week's rehearsal was missed",
                    event.points
                ),
            )
        } else {
            (
                -(event.points as f64),
                "Full deduction for unexcused absence from this week's rehearsal".to_owned(),
            )
        }
    }

    fn attended_bonus_event(event: &Event, grade: f64) -> (f64, String) {
        // Get back points for volunteer gigs and and extra sectionals and ombuds events
        if grade + event.points as f64 > 100.0 {
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
                event.points as f64,
                "Full bonus awarded for attending volunteer or extra event".to_owned(),
            )
        }
    }

    fn should_have_attended(
        event: &EventWithAttendance,
        week: &WeekOfAttendances,
        now: &OffsetDateTime,
    ) -> (f64, String) {
        // Lose the full point value if did not attend
        if event.event.r#type == EventType::OMBUDS {
            (
                0.0,
                "You do not lose points for missing an ombuds event".to_owned(),
            )
        } else if &event.event.r#type == EventType::SECTIONAL
            && week
                .events_of_type(EventType::SECTIONAL)
                .any(|event| event.did_attend())
        {
            (
                0.0,
                "No deduction because you attended a different sectional this week".to_owned(),
            )
        } else if event.event.r#type == EventType::SECTIONAL
            && week
                .missed_event_of_type(EventType::SECTIONAL)
                .map(|missed_sectional| {
                    missed_sectional.event.call_time.0 < event.event.call_time.0
                })
                .unwrap_or(false)
        {
            (
                0.0,
                "No deduction because you already lost points for one sectional this week"
                    .to_owned(),
            )
        } else if event.event.r#type == EventType::SECTIONAL
            && week
                .events_of_type(EventType::SECTIONAL)
                .last()
                .map(|last_sectional| {
                    last_sectional.event.call_time.0 > event.event.call_time.0
                        && &last_sectional.event.call_time.0 > now
                })
                .unwrap_or(false)
        {
            (
                0.0,
                "No deduction because not all sectionals occurred yet".to_owned(),
            )
        } else if event.approved_absence() {
            (
                0.0,
                "No deduction because an absence request was submitted and approved".to_owned(),
            )
        } else {
            (
                -(event.event.points as f64),
                "Full deduction for unexcused absence from event".to_owned(),
            )
        }
    }
}
