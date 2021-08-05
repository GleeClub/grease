use async_graphql::{ComplexObject, Context, Enum};

#[derive(ComplexObject)]
pub struct ActiveSemester {
    /// The email of the member
    pub member: String,
    /// The name of the semester
    pub semester: String,
    /// Whether the member was registered for the class
    pub enrollment: Enrollment,
    /// What section the member sang in
    pub section: Option<String>,
}

#[complex]
impl ActiveSemester {
    /// The grades for the member in the given semester
    pub async fn grades(&self, ctx: &Context) -> Result<Grades> {
        // Grades.for_member (Member.with_email! @member), (Semester.with_name! @semester)
    }
}

#[derive(Enum)]
pub enum Enrollment {
    Class,
    Club,
}

// impl ActiveSemester {

// module Models
//   @[GraphQL::Object]
//   class ActiveSemester
//     include GraphQL::ObjectType

//     class_getter table_name = "active_semester"

//     DB.mapping({
//       member:     String,
//       semester:   String,
//       enrollment: {type: Enrollment, default: Enrollment::CLUB, converter: Enrollment},
//       section:    String?,
//     })

//     def self.all_for_member(email)
//       CONN.query_all "SELECT * FROM #{@@table_name} WHERE member = ?", email, as: ActiveSemester
//     end

//     def self.for_semester(email, semester_name)
//       CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ? AND SEMESTER = ?",
//         email, semester_name, as: ActiveSemester
//     end

//     def self.create_for_member(member, form, semester)
//       if member.get_semester(semester.name)
//         raise "#{member.full_name} is already active for the current semester"
//       end

//       CONN.exec "INSERT INTO #{@@table_name} (member, semester, enrollment, section)
//         VALUES (?, ?, ?, ?)", member.email, semester.name, form.enrollment.to_rs, form.section
//     end

//     def self.update(email, semester_name, enrollment, section)
//       active_semester = for_semester email, semester_name

//       if enrollment
//         if active_semester
//           CONN.exec "UPDATE #{@@table_name} SET enrollment = ?, section = ? \
//             WHERE member = ? AND semester = ?", enrollment.to_rs, section, email, semester_name
//         else
//           CONN.exec "INSERT INTO #{@@table_name} (member, semester, enrollment, section)
//             VALUES (?, ?, ?, ?)", email, semester_name, enrollment.to_rs, section
//         end
//       elsif active_semester
//         CONN.exec "DELETE FROM #{@@table_name} WHERE member = ? AND SEMESTER = ?", email, semester_name
//       end
//     end
