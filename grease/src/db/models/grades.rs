use chrono::{Datelike, Duration, Local, NaiveDateTime};
use db::models::attendance::MemberAttendance;
use db::{ActiveSemester, Attendance, Event, Member, Semester};
use diesel::prelude::*;
use error::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Grades {
    pub grade: f32,
    #[serde(rename = "eventsWithChanges")]
    pub events_with_changes: Vec<EventWithGradeChange>,
    #[serde(rename = "volunteerGigsAttended")]
    pub volunteer_gigs_attended: usize,
}

#[derive(Serialize)]
pub struct EventWithGradeChange {
    #[serde(flatten)]
    pub event: MemberAttendance,
    pub change: GradeChange,
}

#[derive(Serialize)]
pub struct GradeChange {
    pub reason: String,
    pub change: f32,
    #[serde(rename = "partialScore")]
    pub partial_score: f32,
}

impl Grades {
    pub fn for_member(
        member: &Member,
        active_semester: Option<&ActiveSemester>,
        semester: &Semester,
        conn: &MysqlConnection,
    ) -> GreaseResult<Grades> {
        let semester_attendance = Attendance::load_for_member_at_all_events(
            &member,
            active_semester.is_some(),
            &semester.name,
            conn,
        )?;

        let initial_grades = Grades {
            grade: 100.0,
            volunteer_gigs_attended: 0,
            events_with_changes: vec![],
        };
        let weeks = Self::organize_into_weeks(&semester, semester_attendance);
        let grades = weeks.fold(initial_grades, |weeks_grades, week| {
            let context = WeeklyAttendanceContext::for_week(&week);

            week.into_iter()
                .fold(weeks_grades, |mut grades, member_attendance| {
                    let change =
                        Self::calculate_grade_change(&member_attendance, &context, grades.grade);
                    let attended_volunteer_gig =
                        Self::attended_volunteer_gig(&member_attendance, &context);

                    grades.grade = change.partial_score;
                    grades.volunteer_gigs_attended += if attended_volunteer_gig { 1 } else { 0 };
                    grades.events_with_changes.push(EventWithGradeChange {
                        event: member_attendance,
                        change,
                    });

                    grades
                })
        });

        Ok(grades)
    }

    fn organize_into_weeks(
        semester: &Semester,
        mut attendance: Vec<MemberAttendance>,
    ) -> impl Iterator<Item = Vec<MemberAttendance>> {
        let first_sunday = Self::first_sunday_of_semester(attendance.first(), semester);
        let end_of_semester = attendance
            .last()
            .map(|a| &a.event.event.call_time)
            .unwrap_or(&semester.end_date)
            .clone();
        let sundays = std::iter::successors(Some(first_sunday), move |sunday| {
            Some(*sunday + Duration::weeks(1)).filter(|next_sunday| *next_sunday <= end_of_semester)
        });

        sundays.map(move |sunday| {
            let following_sunday = sunday + Duration::weeks(1);
            attendance
                .drain_filter(|member_attendance| {
                    let call_time = &member_attendance.event.event.call_time;
                    call_time > &sunday && call_time <= &following_sunday
                })
                .collect()
        })
    }

    fn first_sunday_of_semester(
        first_event: Option<&MemberAttendance>,
        semester: &Semester,
    ) -> NaiveDateTime {
        let beginning_of_semester = first_event
            .map(|MemberAttendance { event, .. }| &event.event.call_time)
            .unwrap_or(&semester.start_date);
        let days_since_sunday = beginning_of_semester
            .date()
            .weekday()
            .num_days_from_sunday() as i64;

        *beginning_of_semester - Duration::days(days_since_sunday)
    }

    fn calculate_grade_change(
        member_attendance: &MemberAttendance,
        context: &WeeklyAttendanceContext,
        grade: f32,
    ) -> GradeChange {
        let event = &member_attendance.event.event;
        let is_bonus_event = Self::is_bonus_event(&member_attendance, &context);

        let (change, reason) = if event.call_time > Local::now().naive_utc() {
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

    fn is_bonus_event(
        member_attendance: &MemberAttendance,
        context: &WeeklyAttendanceContext,
    ) -> bool {
        let event = &member_attendance.event.event;

        ([Event::VOLUNTEER_GIG, Event::OMBUDS].contains(&event.type_.as_str()))
            || (event.type_ == Event::OTHER && !member_attendance.should_attend())
            || (event.type_ == Event::SECTIONAL && context.missed_sectional.is_none())
    }

    fn attended_volunteer_gig(
        member_attendance: &MemberAttendance,
        context: &WeeklyAttendanceContext,
    ) -> bool {
        let event = &member_attendance.event.event;

        member_attendance.did_attend()
            && !context.missed_rehearsal
            && event.type_ == Event::VOLUNTEER_GIG
            && event.gig_count
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
        member_attendance: &MemberAttendance,
        grade: f32,
        bonus_event: bool,
    ) -> (f32, String) {
        let event = &member_attendance.event.event;
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
        } else if member_attendance.should_attend() {
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

    fn points_lost_for_lateness(event: &Event, minutes_late: i32) -> f32 {
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

struct WeeklyAttendanceContext {
    pub missed_rehearsal: bool,
    pub missed_sectional: Option<NaiveDateTime>,
    pub attended_sectionals: bool,
    pub last_sectional: Option<NaiveDateTime>,
}

impl WeeklyAttendanceContext {
    pub fn for_week(week: &Vec<MemberAttendance>) -> Self {
        Self {
            missed_rehearsal: Self::missed_rehearsal_during_week(week),
            missed_sectional: Self::missed_sectional_during_week(week),
            attended_sectionals: Self::attended_sectional_during_week(week),
            last_sectional: Self::last_sectional_call_time(week),
        }
    }

    fn missed_rehearsal_during_week(week: &Vec<MemberAttendance>) -> bool {
        week.iter()
            .filter(|member_attendance| {
                member_attendance.event.event.type_ == Event::REHEARSAL
                    && member_attendance.deny_credit()
            })
            .next()
            .is_some()
    }

    /// returns the first sectional's call time that was missed that week,
    /// or None is all were attended.
    fn missed_sectional_during_week(week: &Vec<MemberAttendance>) -> Option<NaiveDateTime> {
        week.iter()
            .filter(|member_attendance| {
                member_attendance.event.event.type_ == Event::SECTIONAL
                    && member_attendance.deny_credit()
            })
            .map(|MemberAttendance { event, .. }| event.event.call_time)
            .next()
    }

    fn attended_sectional_during_week(week: &Vec<MemberAttendance>) -> bool {
        week.iter()
            .filter(|member_attendance| {
                member_attendance.event.event.type_ == Event::SECTIONAL
                    && member_attendance.did_attend()
            })
            .map(|MemberAttendance { event, .. }| (event.event.id, &event.event.call_time))
            .next()
            .is_some()
    }

    fn last_sectional_call_time(week: &Vec<MemberAttendance>) -> Option<NaiveDateTime> {
        week.iter()
            .filter(|member_attendance| member_attendance.event.event.type_ == Event::SECTIONAL)
            .map(|member_attendance| member_attendance.event.event.call_time)
            .last()
    }
}
