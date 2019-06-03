use db::models::*;
use db::schema::member::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use crate::error::{GreaseError, GreaseResult};
use crate::db::schema::enums::*;

impl Member {
    pub fn load(given_email: &str, conn: &MysqlConnection) -> GreaseResult<Member> {
        member
            .filter(email.eq(given_email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!("no member exists with the email {}", given_email)))
    }

    // TODO: make this one query
    pub fn load_from_token(grease_token: &str, conn: &MysqlConnection) -> GreaseResult<Option<Member>> {
        if let Some(member_session) = session::dsl::session
            .filter(session::dsl::key.eq(grease_token))
            .first::<Session>(conn)
            .optional()
            .map_err(GreaseError::DbError)? {
            Member::load(&member_session.member, &conn).map(|m| Some(m))
        } else {
            Ok(None)
        }
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Member>> {
        member
            .order(first_name)
            .order(last_name)
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(new_member: Member, conn: &MysqlConnection) -> GreaseResult<String> {
        if let Ok(existing_member) = Self::load(&new_member.email, conn) {
            Err(GreaseError::BadRequest(format!(
                "A member with the email {} already exists.",
                existing_member.email
            )))
        } else {
            let new_member_email: String = diesel::insert_into(member)
                .values(new_member)
                .returning(email)
                .get_result(conn)
                .map_err(GreaseError::DbError)?;

            Attendance::create_for_new_member(&new_member_email, conn)?;
            Ok(new_member_email)
        }
    }

    pub fn full_name(&self) -> String {
        if let Some(preferred_name) = self.preferred_name {
            format!(
                "{} \"{}\" {}",
                self.first_name, preferred_name, self.last_name
            )
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }

    pub fn num_volunteer_gigs(&self, conn: &MysqlConnection) -> usize {
        Attendance::load_for_member_at_all_events_of_type(
            &self.email,
            "volunteer",
            conn,
        ).iter()
            .filter(|(a, _e)| a.did_attend)
            .count()
    }

    // returns a tuple of: 1) A vec with events, excuses, and grade changes, and 2) the final grade
    pub fn calc_grades(&self, conn: &MysqlConnection) -> GreaseResult<Grades> {
        let mut grade = 100.0;
        let mut grade_items = Vec::new();
        let mut events = Event::load_all(conn)?;
        events.sort_by_key(|e| e.start_time);

        while let Some(event) = events.pop() {
            let old_grade = grade;
            let (new_grade, reason) = event.grade(self, old_grade, conn);
            grade = new_grade;
            grade_items.push(GradeChange {
                event,
                reason,
                change: grade - old_grade,
            });
        }

        Ok(Grades {
            final_grade: grade,
            changes: grade_items,
        })
    }

    pub fn has_permission(&self, permission: &str, conn: &MysqlConnection) -> bool {
        self.permissions(conn)
            .iter()
            .find(|&p| p == permission)
            .is_some()
    }

    pub fn permissions(&self, conn: &MysqlConnection) -> GreaseResult<Vec<String>> {
        match self.officer_pos {
            // TODO: load in officer permissions, remove "placeholder"
            Some(position) => Ok(vec!["placeholder"]),
            Some(position) => Ok(Vec::new()),
        }
    }
}

pub struct Grades {
    pub final_grade: f32,
    pub changes: Vec<GradeChange>,
}

pub struct GradeChange {
    pub event: Event,
    pub reason: String,
    pub change: f32,
}

// impl PublicJson for Member {
//     fn public_json(&self, conn: &MysqlConnection) -> Value {
//         let (changes, grade) = self.calc_grades(conn);
//         json!({
//             "email": self.email,
//             "name": self.full_name(),
//             "first_name": self.first_name,
//             "nick_name": self.nick_name,
//             "last_name": self.last_name,
//             "section": self.section,
//             "phone_number": self.phone_number,
//             "location": self.location,
//             "in_class": self.in_class,
//             "active": self.active,
//             "is_driver": self.is_driver,
//             "num_seats": self.num_seats,
//             "enrollment": self.enrollment(),
//             "year_at_tech": self.year_at_tech,
//             "officer_pos": self.officer_pos,
//             "permissions": self.permissions(conn),
//             "num_volunteer_gigs": self.num_volunteer_gigs(conn),
//             "grade_changes": changes,
//             "major": self.major,
//             "grade": grade,
//         })
//     }
// }

// #[derive(Queryable, Identifiable)]
// #[table_name = "member"]
// #[primary_key(email)]
// pub struct NewMember {
//     pub email: String,
//     pub first_name: String,
//     pub preferred_name: Option<String>,
//     pub last_name: String,
//     pub pass_hash: String,
//     pub phone_number: String,
//     pub passengers: i32,
//     pub location: String,
//     pub about: Option<String>,
//     pub major: Option<String>,
//     pub minor: Option<String>,
//     pub hometown: Option<String>,
//     pub arrived_at_tech: Option<i32>,
//     pub gateway_drug: Option<String>,
//     pub conflicts: Option<String>,
//     pub dietary_restrictions: Option<String>,
// }

// #[primary_key(member, semester)]
// #[belongs_to(Member, foreign_key = "member")]
// #[belongs_to(Semester, foreign_key = "semester")]
// pub struct ActiveSemester {
//     pub member: String,
//     pub semester: i32,
//     pub enrollment: Enrollment,
//     pub section: Option<i32>,
// }
