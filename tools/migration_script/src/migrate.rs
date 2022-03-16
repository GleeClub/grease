use std::collections::HashMap;

use bcrypt::hash;
use mysql::Pool;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use crate::error::*;
use crate::new_schema::*;
use crate::old_schema::*;

pub trait Load: Sized {
    fn load(old_db: &Pool) -> MigrateResult<Vec<Self>>;
}

pub trait Insert: Sized {
    fn insert(new_db: &Pool, new_value: &Vec<Self>) -> MigrateResult<()>;
}

pub trait Migrate<Old: Load>: Insert {
    type Dependencies;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<Old>, Vec<Self>)>;
}

impl Migrate<OldMember> for NewMember {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldMember>, Vec<Self>)> {
        let old_members = OldMember::load(old_db)?;
        let new_members = old_members
            .iter()
            .map(|old_member| {
                Ok(NewMember {
                    email: old_member.email.clone(),
                    first_name: old_member.firstName.clone().ok_or(MigrateError::Other(
                        "old member had no first name".to_owned(),
                    ))?,
                    preferred_name: old_member.prefName.clone(),
                    last_name: old_member.lastName.clone().ok_or(MigrateError::Other(
                        "old member had no last name".to_owned(),
                    ))?,
                    pass_hash: old_member
                        .password
                        .clone()
                        .ok_or(MigrateError::Other("old member had no password".to_owned()))
                        .and_then(|pass_hash| {
                            hash(&pass_hash, 10).map_err(|err| {
                                MigrateError::Other(format!("Invalid password hash: {}", err,))
                            })
                        })?,
                    phone_number: old_member
                        .phone
                        .ok_or(MigrateError::Other(
                            "old member had no phone number".to_owned(),
                        ))?
                        .to_string(),
                    picture: old_member.picture.clone(),
                    passengers: std::cmp::max(old_member.passengers, 0),
                    location: old_member
                        .location
                        .as_ref()
                        .map(|location| location.clone())
                        .unwrap_or("".to_owned()),
                    on_campus: old_member.onCampus.clone(),
                    about: old_member.about.clone(),
                    major: old_member.major.clone(),
                    minor: old_member.minor.clone(),
                    hometown: old_member.hometown.clone(),
                    arrived_at_tech: old_member
                        .techYear
                        .clone()
                        .map(|year| std::cmp::max(year, 1)),
                    gateway_drug: old_member.gatewayDrug.clone(),
                    conflicts: old_member.conflicts.clone(),
                    dietary_restrictions: None,
                })
            })
            .collect::<MigrateResult<Vec<NewMember>>>()?;
        Insert::insert(new_db, &new_members)?;

        Ok((old_members, new_members))
    }
}

impl Migrate<OldSemester> for NewSemester {
    type Dependencies = Vec<OldVariables>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldSemester>, Vec<Self>)> {
        let old_semesters = OldSemester::load(old_db)?;
        let current_semester = dependencies
            .iter()
            .next()
            .ok_or(MigrateError::Other(
                "No variables set in old table".to_owned(),
            ))?
            .semester
            .clone();
        let new_semesters = old_semesters
            .iter()
            .map(|old_semester| NewSemester {
                name: old_semester.semester.clone(),
                start_date: old_semester.beginning.clone(),
                end_date: old_semester.end.clone(),
                gig_requirement: old_semester.gigreq,
                current: old_semester.semester == current_semester,
            })
            .collect::<Vec<NewSemester>>();

        if !new_semesters
            .iter()
            .any(|new_semester| new_semester.current)
        {
            Err(MigrateError::Other(format!(
                "no semester was current, the current semester was named {:?}",
                current_semester
            )))
        } else {
            Insert::insert(new_db, &new_semesters)?;
            Ok((old_semesters, new_semesters))
        }
    }
}

impl Migrate<OldRole> for NewRole {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldRole>, Vec<Self>)> {
        let old_roles = OldRole::load(old_db)?;
        let new_roles = old_roles
            .iter()
            .map(|old_role| {
                Ok(NewRole {
                    name: old_role
                        .name
                        .as_ref()
                        .ok_or(MigrateError::Other(format!(
                            "old role with id {} had no name",
                            old_role.id
                        )))?
                        .clone(),
                    rank: if old_role.rank < 0 {
                        100
                    } else {
                        old_role.rank
                    },
                    max_quantity: std::cmp::max(old_role.quantity, 0),
                })
            })
            .collect::<MigrateResult<Vec<NewRole>>>()?;
        Insert::insert(new_db, &new_roles)?;

        Ok((old_roles, new_roles))
    }
}

impl Migrate<OldMemberRole> for NewMemberRole {
    type Dependencies = Vec<OldRole>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldMemberRole>, Vec<Self>)> {
        let old_member_roles = OldMemberRole::load(old_db)?;
        let new_member_roles = old_member_roles
            .iter()
            .map(|old_member_role| {
                Ok(NewMemberRole {
                    member: old_member_role.member.clone(),
                    role: dependencies
                        .iter()
                        .find(|role| role.id == old_member_role.role)
                        .ok_or(MigrateError::Other(format!(
                            "no old role had the id {}",
                            old_member_role.role
                        )))?
                        .name
                        .as_ref()
                        .ok_or(MigrateError::Other(format!(
                            "role with id {} had no name",
                            old_member_role.role
                        )))?
                        .clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewMemberRole>>>()?;
        Insert::insert(new_db, &new_member_roles)?;

        Ok((old_member_roles, new_member_roles))
    }
}

impl Migrate<OldSectionType> for NewSectionType {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldSectionType>, Vec<Self>)> {
        let old_section_types = OldSectionType::load(old_db)?;
        let new_section_types = old_section_types
            .iter()
            .filter(|old_section_type| old_section_type.choir == Some("glee".to_owned()))
            .map(|old_section_type| {
                Ok(NewSectionType {
                    name: old_section_type
                        .name
                        .as_ref()
                        .ok_or(MigrateError::Other(format!(
                            "old section type with id {} had no name",
                            old_section_type.id
                        )))?
                        .clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewSectionType>>>()?;
        if new_section_types.len() == 0 {
            return Err(MigrateError::Other(
                "No section types for Glee Club found".to_owned(),
            ));
        }

        Insert::insert(new_db, &new_section_types)?;

        Ok((old_section_types, new_section_types))
    }
}

impl Migrate<OldEventType> for NewEventType {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldEventType>, Vec<Self>)> {
        let old_event_types = OldEventType::load(old_db)?;
        let new_event_types = old_event_types
            .iter()
            .map(|old_event_type| NewEventType {
                name: old_event_type.name.clone(),
                weight: old_event_type.weight,
            })
            .collect::<Vec<NewEventType>>();
        Insert::insert(new_db, &new_event_types)?;

        Ok((old_event_types, new_event_types))
    }
}

impl Migrate<OldEvent> for NewEvent {
    type Dependencies = (Vec<OldSectionType>, Vec<OldEventType>);
    fn migrate<'a>(
        old_db: &Pool,
        new_db: &Pool,
        (old_section_types, old_event_types): &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldEvent>, Vec<Self>)> {
        let old_events = OldEvent::load(old_db)?;
        let new_events = old_events
            .iter()
            .map(|old_event| {
                Ok(NewEvent {
                    id: old_event.eventNo.clone(),
                    name: old_event.name.clone(),
                    semester: old_event.semester.clone(),
                    call_time: old_event.callTime.clone(),
                    release_time: old_event.releaseTime.clone(),
                    points: old_event.points.clone(),
                    comments: old_event.comments.clone(),
                    location: old_event.location.clone(),
                    gig_count: old_event.gigcount.clone(),
                    default_attend: old_event.defaultAttend.clone(),
                    type_: old_event_types
                        .iter()
                        .find(|event_type| event_type.id == old_event.type_)
                        .ok_or(MigrateError::Other(format!(
                            "event with id {} had an unknown event type {}",
                            old_event.eventNo, old_event.type_,
                        )))?
                        .name
                        .clone(),
                    section: if old_event.section == 0 {
                        None
                    } else {
                        let section_type = old_section_types
                            .iter()
                            .find(|section_type| section_type.id == old_event.section)
                            .ok_or(MigrateError::Other(format!(
                                "event with id {} had a section with unknown id {}",
                                old_event.eventNo, old_event.section,
                            )))?;

                        Some(
                            section_type
                                .name
                                .as_ref()
                                .ok_or(MigrateError::Other(format!(
                                    "event with id {} had a section with no name (id {})",
                                    old_event.eventNo, old_event.section,
                                )))?
                                .clone(),
                        )
                    },
                })
            })
            .collect::<MigrateResult<Vec<NewEvent>>>()?;
        Insert::insert(new_db, &new_events)?;

        Ok((old_events, new_events))
    }
}

impl Migrate<OldAbsenceRequest> for NewAbsenceRequest {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldAbsenceRequest>, Vec<Self>)> {
        let old_absence_requests = OldAbsenceRequest::load(old_db)?;
        let new_absence_requests = old_absence_requests
            .iter()
            .map(|old_absence_request| NewAbsenceRequest {
                member: old_absence_request.memberID.clone(),
                event: old_absence_request.eventNo,
                time: old_absence_request.time.clone(),
                reason: old_absence_request.reason.clone(),
                state: if old_absence_request.state == "confirmed" {
                    "approved".to_owned()
                } else {
                    old_absence_request.state.clone()
                },
            })
            .collect::<Vec<NewAbsenceRequest>>();
        Insert::insert(new_db, &new_absence_requests)?;

        Ok((old_absence_requests, new_absence_requests))
    }
}

impl Migrate<OldActiveSemester> for NewActiveSemester {
    type Dependencies = Vec<OldSectionType>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldActiveSemester>, Vec<Self>)> {
        let old_active_semesters = OldActiveSemester::load(old_db)?;
        let new_active_semesters = old_active_semesters
            .iter()
            .filter(|old_active_semester| old_active_semester.choir == "glee")
            .map(|old_active_semester| {
                Ok(NewActiveSemester {
                    member: old_active_semester.member.clone(),
                    semester: old_active_semester.semester.clone(),
                    enrollment: old_active_semester.enrollment.clone(),
                    section: if old_active_semester.section == 0 {
                        None
                    } else {
                        let section_type = dependencies
                            .iter()
                            .find(|section_type| section_type.id == old_active_semester.section)
                            .ok_or(MigrateError::Other(format!(
                                "active semester for member {} during semester {} had a section with unknown id {}",
                                old_active_semester.member, old_active_semester.semester, old_active_semester.section,
                            )))?;

                        if section_type.choir != Some("glee".to_owned()) {
                            return Err(MigrateError::Other(format!(
                                "active semester for member {} during semester {} had a section for the wrong choir ({:?})",
                                old_active_semester.member, old_active_semester.semester, section_type.choir,
                            )))
                        }

                        Some(section_type.name.as_ref().ok_or(MigrateError::Other(format!(
                            "active semester for member {} during semester {} had a section with no name (id {})",
                            old_active_semester.member, old_active_semester.semester, old_active_semester.section,
                        )))?.clone())
                    },
                })
            })
            .collect::<MigrateResult<Vec<NewActiveSemester>>>()?;
        if new_active_semesters.len() == 0 {
            return Err(MigrateError::Other(
                "no active semesters for the Glee Club".to_owned(),
            ));
        }

        Insert::insert(new_db, &new_active_semesters)?;

        Ok((old_active_semesters, new_active_semesters))
    }
}

impl Migrate<OldAnnouncement> for NewAnnouncement {
    type Dependencies = Vec<OldSemester>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldAnnouncement>, Vec<Self>)> {
        let old_announcements = OldAnnouncement::load(old_db)?;
        let mut old_semesters = (*dependencies).clone();
        old_semesters.sort_by_key(|old_semester| old_semester.beginning);

        let new_announcements = old_announcements
            .iter()
            .map(|old_announcement| {
                Ok(NewAnnouncement {
                    id: old_announcement.announcementNo,
                    member: Some(old_announcement.memberID.clone()),
                    time: old_announcement.timePosted.clone(),
                    content: old_announcement.announcement.clone(),
                    archived: old_announcement.archived,
                    semester: old_semesters.iter().take_while(|old_semester| {
                        old_semester.beginning < old_announcement.timePosted
                    })
                    .next()
                    .ok_or(MigrateError::Other(
                        format!(
                            "announcement not made during any previously existing semester (posted on {})",
                            old_announcement.timePosted.format("%c")
                        ))
                    )?
                    .semester.clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewAnnouncement>>>()?;
        Insert::insert(new_db, &new_announcements)?;

        Ok((old_announcements, new_announcements))
    }
}

impl Migrate<OldAttends> for NewAttendance {
    type Dependencies = (Vec<OldActiveSemester>, Vec<OldEvent>);
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        (old_active_semesters, old_events): &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldAttends>, Vec<Self>)> {
        let old_attendances = OldAttends::load(old_db)?;
        let mut grouped_active_semesters = old_active_semesters.iter().fold(
            HashMap::new(),
            |mut map: HashMap<String, Vec<&OldActiveSemester>>, active_semester| {
                map.get_mut(&active_semester.semester)
                    .get_or_insert(&mut Vec::new())
                    .push(active_semester);
                map
            },
        );
        let grouped_events = old_events.iter().fold(
            HashMap::new(),
            |mut map: HashMap<String, Vec<&OldEvent>>, event| {
                map.get_mut(&event.semester)
                    .get_or_insert(&mut Vec::new())
                    .push(event);
                map
            },
        );

        let mut new_attendances = old_attendances
            .iter()
            .map(|old_attendance| NewAttendance {
                member: old_attendance.memberID.clone(),
                event: old_attendance.eventNo,
                should_attend: old_attendance.shouldAttend,
                did_attend: old_attendance.didAttend.unwrap_or(false),
                confirmed: old_attendance.confirmed,
                minutes_late: old_attendance.minutesLate,
            })
            .collect::<Vec<NewAttendance>>();

        let mut additional_new_attendances: Vec<NewAttendance> = Vec::new();
        for (semester, events) in grouped_events.iter() {
            for event in events {
                let active_semesters =
                    grouped_active_semesters
                        .remove(semester)
                        .ok_or(MigrateError::Other(format!(
                        "event {} was during semester '{}' which no active semesters were found in",
                        event.eventNo, semester
                    )))?;
                additional_new_attendances.extend(active_semesters.into_iter().filter_map(
                    |active_semester| {
                        if !new_attendances.iter().any(|attendance| {
                            attendance.member == active_semester.member
                                && attendance.event == event.eventNo
                        }) {
                            Some(NewAttendance {
                                member: active_semester.member.clone(),
                                event: event.eventNo,
                                should_attend: false,
                                did_attend: false,
                                confirmed: false,
                                minutes_late: 0,
                            })
                        } else {
                            None
                        }
                    },
                ));
            }
        }

        new_attendances.extend(additional_new_attendances.into_iter());
        Insert::insert(new_db, &new_attendances)?;

        Ok((old_attendances, new_attendances))
    }
}

impl Migrate<OldCarpool> for NewCarpool {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldCarpool>, Vec<Self>)> {
        let old_carpools = OldCarpool::load(old_db)?;
        let new_carpools = old_carpools
            .iter()
            .map(|old_carpool| NewCarpool {
                id: old_carpool.carpoolID,
                driver: old_carpool.driver.clone(),
                event: old_carpool.eventNo,
            })
            .collect::<Vec<NewCarpool>>();
        Insert::insert(new_db, &new_carpools)?;

        Ok((old_carpools, new_carpools))
    }
}

impl Migrate<OldFee> for NewFee {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldFee>, Vec<Self>)> {
        let old_fees = OldFee::load(old_db)?;
        let new_fees = old_fees
            .iter()
            .map(|old_fee| {
                Ok(NewFee {
                    amount: old_fee.amount,
                    name: old_fee.id.clone(),
                    description: old_fee
                        .name
                        .as_ref()
                        .ok_or(MigrateError::Other(format!(
                            "fee with id {} had no name/description",
                            old_fee.id
                        )))?
                        .clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewFee>>>()?;
        Insert::insert(new_db, &new_fees)?;

        Ok((old_fees, new_fees))
    }
}

impl Migrate<OldGDocs> for NewGoogleDocs {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldGDocs>, Vec<Self>)> {
        let old_gdocs = OldGDocs::load(old_db)?;
        let new_google_docs = old_gdocs
            .iter()
            .map(|old_gdoc| NewGoogleDocs {
                name: old_gdoc.name.clone(),
                url: old_gdoc.url.clone(),
            })
            .collect::<Vec<NewGoogleDocs>>();
        Insert::insert(new_db, &new_google_docs)?;

        Ok((old_gdocs, new_google_docs))
    }
}

impl Migrate<OldUniform> for NewUniform {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldUniform>, Vec<Self>)> {
        let old_uniforms = OldUniform::load(old_db)?;
        let new_uniforms = old_uniforms
            .iter()
            .filter(|old_uniform| old_uniform.choir == "glee")
            .enumerate()
            .map(|(index, old_uniform)| NewUniform {
                id: index as i64 + 1,
                name: old_uniform.name.clone(),
                description: Some(old_uniform.description.clone()).filter(|d| d.len() > 0),
                color: match old_uniform.id.as_str() {
                    "casual" => Some("#a8c".to_owned()),
                    "jeans" | "tshirt_mode" => Some("#137".to_owned()),
                    "slacks" | "wedding" => Some("#000".to_owned()),
                    "tshirt" => Some("#dc3".to_owned()),
                    _other => None,
                },
            })
            .collect::<Vec<NewUniform>>();
        Insert::insert(new_db, &new_uniforms)?;

        Ok((old_uniforms, new_uniforms))
    }
}

impl Migrate<OldGig> for NewGig {
    type Dependencies = (Vec<OldUniform>, Vec<NewUniform>);
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        (old_uniforms, new_uniforms): &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldGig>, Vec<Self>)> {
        let old_gigs = OldGig::load(old_db)?;
        let new_gigs = old_gigs
            .iter()
            .map(|old_gig| {
                Ok(NewGig {
                    event: old_gig.eventNo,
                    performance_time: old_gig.performanceTime.clone(),
                    contact_name: old_gig.cname.clone(),
                    contact_email: old_gig.cemail.clone(),
                    contact_phone: old_gig.cphone.clone(),
                    price: old_gig.price.clone(),
                    public: old_gig.public.clone(),
                    summary: Some(old_gig.summary.clone()),
                    description: Some(old_gig.description.clone()),
                    uniform: {
                        let old_uniform = old_uniforms
                            .iter()
                            .find(|old_uniform| old_uniform.id == old_gig.uniform)
                            .ok_or(MigrateError::Other(format!(
                                "no uniform with id {}",
                                old_gig.uniform
                            )))?;
                        let new_uniform = new_uniforms
                            .iter()
                            .find(|new_uniform| new_uniform.name == old_uniform.name)
                            .ok_or(MigrateError::Other(format!(
                                "no new uniform with name {}",
                                &old_uniform.name
                            )))?;
                        new_uniform.id
                    },
                })
            })
            .collect::<MigrateResult<Vec<NewGig>>>()?;
        Insert::insert(new_db, &new_gigs)?;

        Ok((old_gigs, new_gigs))
    }
}

impl Migrate<OldGigReq> for NewGigRequest {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldGigReq>, Vec<Self>)> {
        let old_gig_requests = OldGigReq::load(old_db)?;
        let new_gig_requests = old_gig_requests
            .iter()
            .map(|old_gig_request| NewGigRequest {
                id: old_gig_request.id,
                time: old_gig_request.timestamp.clone(),
                name: old_gig_request.name.clone(),
                organization: old_gig_request.org.clone(),
                event: old_gig_request.eventNo.clone(),
                contact_name: old_gig_request.cname.clone(),
                contact_email: old_gig_request.cemail.clone(),
                contact_phone: old_gig_request.cphone.clone(),
                start_time: old_gig_request.startTime.clone(),
                location: old_gig_request.location.clone(),
                comments: Some(old_gig_request.comments.clone()),
                status: old_gig_request.status.clone(),
            })
            .collect::<Vec<NewGigRequest>>();
        Insert::insert(new_db, &new_gig_requests)?;

        Ok((old_gig_requests, new_gig_requests))
    }
}

impl Migrate<OldSong> for NewSong {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldSong>, Vec<Self>)> {
        let map_key = |old_key, song_id| match old_key {
            "A♭" | "a♭" => Ok(Some("a_flat")),
            "A" | "a" => Ok(Some("a")),
            "A♯" | "a♯" => Ok(Some("a_sharp")),
            "B♭" | "b♭" => Ok(Some("b_flat")),
            "B" | "b" => Ok(Some("b")),
            "B♯" | "b♯" => Ok(Some("b_sharp")),
            "C♭" | "c♭" => Ok(Some("c_flat")),
            "C" | "c" => Ok(Some("c")),
            "C♯" | "c♯" => Ok(Some("c_sharp")),
            "D♭" | "d♭" => Ok(Some("d_flat")),
            "D" | "d" => Ok(Some("d")),
            "D♯" | "d♯" => Ok(Some("d_sharp")),
            "E♭" | "e♭" => Ok(Some("e_flat")),
            "E" | "e" => Ok(Some("e")),
            "E♯" | "e♯" => Ok(Some("e_sharp")),
            "F♭" | "f♭" => Ok(Some("f_flat")),
            "F" | "f" => Ok(Some("f")),
            "F♯" | "f♯" => Ok(Some("f_sharp")),
            "G♭" | "g♭" => Ok(Some("g_flat")),
            "G" | "g" => Ok(Some("g")),
            "G♯" | "g♯" => Ok(Some("g_sharp")),
            "?" => Ok(None),
            _other => Err(MigrateError::Other(format!(
                "song with id {} had unknown key/pitch = {:?}",
                song_id, old_key
            ))),
        };
        let old_songs = OldSong::load(old_db)?;
        let new_songs = old_songs
            .iter()
            .map(|old_song| {
                Ok(NewSong {
                    id: old_song.id,
                    title: old_song.title.clone(),
                    info: Some(old_song.info.clone()),
                    current: old_song.current.clone(),
                    key: map_key(&old_song.key, old_song.id)?.map(|key| key.to_owned()),
                    starting_pitch: map_key(&old_song.pitch, old_song.id)?
                        .map(|pitch| pitch.to_owned()),
                    mode: None,
                })
            })
            .collect::<MigrateResult<Vec<NewSong>>>()?;
        Insert::insert(new_db, &new_songs)?;

        Ok((old_songs, new_songs))
    }
}

impl Migrate<OldSongLink> for NewSongLink {
    type Dependencies = Vec<OldMediaType>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldSongLink>, Vec<Self>)> {
        let old_song_links = OldSongLink::load(old_db)?;
        let new_song_links = old_song_links
            .iter()
            .map(|old_song| {
                Ok(NewSongLink {
                    id: old_song.id,
                    song: old_song.song,
                    name: old_song.name.clone(),
                    target: utf8_percent_encode(&old_song.target, DEFAULT_ENCODE_SET).to_string(),
                    type_: dependencies
                        .iter()
                        .find(|old_media_type| old_media_type.typeid == old_song.type_)
                        .ok_or(MigrateError::Other(format!(
                            "song with id {} had an unknown media type {}",
                            old_song.id, old_song.type_
                        )))?
                        .name
                        .clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewSongLink>>>()?;
        Insert::insert(new_db, &new_song_links)?;

        Ok((old_song_links, new_song_links))
    }
}

impl Migrate<OldGigSong> for NewGigSong {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldGigSong>, Vec<Self>)> {
        let old_gig_songs = OldGigSong::load(old_db)?;
        let new_gig_songs = old_gig_songs
            .iter()
            .map(|old_gig_song| NewGigSong {
                event: old_gig_song.event,
                song: old_gig_song.song,
                order: old_gig_song.order,
            })
            .collect::<Vec<NewGigSong>>();
        Insert::insert(new_db, &new_gig_songs)?;

        Ok((old_gig_songs, new_gig_songs))
    }
}

impl Migrate<OldMediaType> for NewMediaType {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldMediaType>, Vec<Self>)> {
        let old_media_types = OldMediaType::load(old_db)?;
        let new_media_types = old_media_types
            .iter()
            .map(|old_media_type| NewMediaType {
                name: old_media_type.name.clone(),
                order: old_media_type.order,
                storage: old_media_type.storage.clone(),
            })
            .collect::<Vec<NewMediaType>>();
        Insert::insert(new_db, &new_media_types)?;

        Ok((old_media_types, new_media_types))
    }
}

impl Migrate<OldMinutes> for NewMinutes {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldMinutes>, Vec<Self>)> {
        let old_minutes = OldMinutes::load(old_db)?;
        let new_minutes = old_minutes
            .iter()
            .map(|old_meeting| NewMinutes {
                id: old_meeting.id,
                name: old_meeting.name.clone(),
                date: old_meeting.date.clone(),
                public: old_meeting.public.clone(),
                private: old_meeting.private.clone(),
            })
            .collect::<Vec<NewMinutes>>();
        Insert::insert(new_db, &new_minutes)?;

        Ok((old_minutes, new_minutes))
    }
}

impl Migrate<OldPermission> for NewPermission {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldPermission>, Vec<Self>)> {
        let old_permissions = OldPermission::load(old_db)?;
        let new_permissions = old_permissions
            .iter()
            .map(|old_permission| NewPermission {
                name: old_permission.name.clone(),
                description: old_permission.description.clone(),
                type_: old_permission.type_.clone(),
            })
            .collect::<Vec<NewPermission>>();
        Insert::insert(new_db, &new_permissions)?;

        Ok((old_permissions, new_permissions))
    }
}

impl Migrate<OldRidesIn> for NewRidesIn {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldRidesIn>, Vec<Self>)> {
        let old_rides_ins = OldRidesIn::load(old_db)?;
        let new_rides_ins = old_rides_ins
            .iter()
            .map(|old_rides_in| NewRidesIn {
                member: old_rides_in.memberID.clone(),
                carpool: old_rides_in.carpoolID,
            })
            .collect::<Vec<NewRidesIn>>();
        Insert::insert(new_db, &new_rides_ins)?;

        Ok((old_rides_ins, new_rides_ins))
    }
}

impl Migrate<OldRolePermission> for NewRolePermission {
    type Dependencies = (Vec<OldRole>, Vec<OldEventType>);
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        (old_roles, old_event_types): &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldRolePermission>, Vec<Self>)> {
        let old_role_permissions = OldRolePermission::load(old_db)?;
        let new_role_permissions = old_role_permissions
            .iter()
            .map(|old_role_permission| {
                Ok(NewRolePermission {
                    id: old_role_permission.id,
                    permission: old_role_permission.permission.clone(),
                    event_type: if let Some(event_type) = &old_role_permission.eventType {
                        Some(
                            old_event_types
                                .iter()
                                .find(|given_event_type| &given_event_type.id == event_type)
                                .ok_or(MigrateError::Other(format!(
                                    "role permission had invalid event type {}",
                                    event_type,
                                )))?
                                .name
                                .clone(),
                        )
                    } else {
                        None
                    },
                    role: old_roles
                        .iter()
                        .find(|old_role| old_role.id == old_role_permission.role)
                        .ok_or(MigrateError::Other(format!(
                            "role permission with id {} has a role of unknown id (id {})",
                            old_role_permission.id, old_role_permission.role,
                        )))?
                        .name
                        .as_ref()
                        .ok_or(MigrateError::Other(format!(
                            "role with id {} has no name",
                            old_role_permission.role
                        )))?
                        .clone(),
                })
            })
            .collect::<MigrateResult<Vec<NewRolePermission>>>()?;
        Insert::insert(new_db, &new_role_permissions)?;

        Ok((old_role_permissions, new_role_permissions))
    }
}

impl Migrate<OldTodo> for NewTodo {
    type Dependencies = Vec<NewMember>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldTodo>, Vec<Self>)> {
        let old_todos = OldTodo::load(old_db)?;
        let mut old_todo_members = OldTodoMembers::load(old_db)?;
        let mut new_todos = Vec::new();
        let mut index = 0;
        for old_todo in &old_todos {
            let todo_members = old_todo_members
                .drain_filter(|todo_member| todo_member.todoID == old_todo.id)
                .map(|todo_member| todo_member.memberID)
                .collect::<Vec<String>>();
            if todo_members.len() == 0 {
                return Err(MigrateError::Other(format!(
                    "todo with id {} had no members",
                    old_todo.id
                )));
            } else {
                new_todos.extend(
                    todo_members
                        .into_iter()
                        .filter(|member| {
                            dependencies
                                .iter()
                                .find(|new_member| &new_member.email == member)
                                .is_some()
                        })
                        .map(|member| NewTodo {
                            id: {
                                index += 1;
                                index
                            },
                            text: old_todo.text.clone(),
                            completed: old_todo.completed,
                            member,
                        }),
                );
            }
        }
        Insert::insert(new_db, &new_todos)?;

        Ok((old_todos, new_todos))
    }
}

impl Migrate<OldTransacType> for NewTransactionType {
    type Dependencies = ();
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldTransacType>, Vec<Self>)> {
        let old_transaction_types = OldTransacType::load(old_db)?;
        let new_transaction_types = old_transaction_types
            .iter()
            .map(|old_transaction_type| NewTransactionType {
                name: old_transaction_type.name.clone(),
            })
            .collect::<Vec<NewTransactionType>>();
        Insert::insert(new_db, &new_transaction_types)?;

        Ok((old_transaction_types, new_transaction_types))
    }
}

impl Migrate<OldTransaction> for NewTransaction {
    type Dependencies = Vec<OldTransacType>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        _dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldTransaction>, Vec<Self>)> {
        let old_transactions = OldTransaction::load(old_db)?;
        let new_transactions = old_transactions
            .iter()
            .map(|old_transaction| NewTransaction {
                id: old_transaction.transactionID,
                member: old_transaction.memberID.clone(),
                time: old_transaction.time.clone(),
                amount: old_transaction.amount,
                description: old_transaction.description.clone(),
                semester: old_transaction.semester.clone(),
                type_: old_transaction.type_.clone(),
                resolved: old_transaction.resolved,
            })
            .collect::<Vec<NewTransaction>>();
        Insert::insert(new_db, &new_transactions)?;

        Ok((old_transactions, new_transactions))
    }
}

impl Migrate<OldVariables> for NewVariable {
    type Dependencies = Vec<OldChoir>;
    fn migrate(
        old_db: &Pool,
        new_db: &Pool,
        dependencies: &Self::Dependencies,
    ) -> MigrateResult<(Vec<OldVariables>, Vec<Self>)> {
        let old_variables = OldVariables::load(old_db)?;
        let variables = if old_variables.len() == 0 {
            return Err(MigrateError::Other(
                "the variables table had no rows".to_owned(),
            ));
        } else if old_variables.len() > 1 {
            return Err(MigrateError::Other(
                "the variables table had multiple rows".to_owned(),
            ));
        } else {
            &old_variables[0]
        };
        let glee_club = dependencies
            .iter()
            .find(|choir| choir.name == "Glee Club")
            .ok_or(MigrateError::Other(
                "Glee Club could not be found in choirs".to_owned(),
            ))?;

        let new_variables = vec![
            NewVariable {
                key: "dues_amount".to_owned(),
                value: variables.duesAmount.to_string(),
            },
            NewVariable {
                key: "tie_deposit".to_owned(),
                value: variables.tieDeposit.to_string(),
            },
            NewVariable {
                key: "late_fee".to_owned(),
                value: variables.lateFee.to_string(),
            },
            NewVariable {
                key: "gig_check".to_owned(),
                value: variables.gigCheck.to_string(),
            },
            NewVariable {
                key: "email_list".to_owned(),
                value: glee_club.list.clone(),
            },
            NewVariable {
                key: "admin_list".to_owned(),
                value: glee_club.admin.clone(),
            },
        ];

        Insert::insert(new_db, &new_variables)?;

        Ok((old_variables, new_variables))
    }
}
