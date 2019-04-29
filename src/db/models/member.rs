use db::models::*;
use db::schema::users::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use serde_json::Value;
use std::fmt;

impl User {
    pub fn load(given_email: &str, conn: &PgConnection) -> Result<User, String> {
        users
            .filter(email.eq(given_email))
            .first(conn)
            .optional()
            .expect("error loading user")
            .ok_or(format!("no user exists with the email {}", given_email))
    }

    pub fn load_all(conn: &PgConnection) -> Vec<User> {
        users
            .order(first_name)
            .order(last_name)
            .load::<User>(conn)
            .expect("error loading all users")
    }

    pub fn create(new_user: &NewUser, conn: &PgConnection) -> Result<String, String> {
        if let Ok(existing_user) = Self::load(&new_user.email, conn) {
            Err(format!(
                "A user with the email {} already exists.",
                existing_user.email
            ))
        } else {
            let new_user_email: String = diesel::insert_into(users)
                .values(new_user)
                .returning(email)
                .get_result(conn)
                .expect("error adding user");

            Attendance::create_for_new_user(&new_user_email, conn);
            Ok(new_user_email)
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

    pub fn num_volunteer_gigs(&self, conn: &PgConnection) -> usize {
        Attendance::load_for_user_at_all_events_of_type(
            &self.email,
            &EventCategory::Volunteer,
            conn,
        ).iter()
            .filter(|(a, _e)| a.did_attend)
            .count()
    }

    // returns a tuple of: 1) A vec with events, excuses, and grade changes, and 2) the final grade
    pub fn calc_grades(&self, conn: &PgConnection) -> (Vec<(Event, String, f32)>, f32) {
        let mut grade = 100.0;
        let mut grade_items = Vec::new();
        let mut events = Event::load_all(conn);
        events.sort_by_key(|e| -e.start_time);

        while let Some(event) = events.pop() {
            let old_grade = grade;
            let (new_grade, reason) = event.grade(self, old_grade, conn);
            grade = new_grade;
            grade_items.push((event, reason, grade - old_grade));
        }

        (grade_items, grade)
    }

    pub fn has_permission(&self, permission: &OfficerPermission, conn: &PgConnection) -> bool {
        self.permissions(conn)
            .iter()
            .find(|&p| p == permission)
            .is_some()
    }

    pub fn permissions(&self, conn: &PgConnection) -> Vec<OfficerPermission> {
        if let Some(ref pos) = self.officer_pos {
            let mut permissions = vec![OfficerPermission::IsOfficer];
            permissions.extend(pos.permissions(conn));
            permissions
        } else {
            Vec::new()
        }
    }
    pub fn enrollment(&self) -> &'static str {
        if !self.active {
            "Inactive"
        } else if self.in_class {
            "Class"
        } else {
            "Club"
        }
    }
}

impl PublicJson for User {
    fn public_json(&self, conn: &PgConnection) -> Value {
        let (changes, grade) = self.calc_grades(conn);
        json!({
            "email": self.email,
            "name": self.full_name(),
            "first_name": self.first_name,
            "nick_name": self.nick_name,
            "last_name": self.last_name,
            "section": self.section,
            "phone_number": self.phone_number,
            "location": self.location,
            "in_class": self.in_class,
            "active": self.active,
            "is_driver": self.is_driver,
            "num_seats": self.num_seats,
            "enrollment": self.enrollment(),
            "year_at_tech": self.year_at_tech,
            "officer_pos": self.officer_pos,
            "permissions": self.permissions(conn),
            "num_volunteer_gigs": self.num_volunteer_gigs(conn),
            "grade_changes": changes,
            "major": self.major,
            "grade": grade,
        })
    }
}

#[derive(Queryable, Identifiable)]
#[table_name = "member"]
#[primary_key(email)]
pub struct Member {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
}

#[primary_key(member, semester, choir)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct ActiveSemester {
    pub member: String,
    pub semester: i32,
    pub choir: String,
    pub enrollment: Enrollment,
    pub section: Option<i32>,
}
