use serde_json::Value;
use warp::reply:json;


from grease.app import app
from datetime import datetime
from flask_login import current_user, login_required
from grease.auth import permission_required, abort_if_not_permitted_to
from flask import request, redirect, render_template, flash, jsonify, abort
from grease.models import EventCategory, Semester, User, UserSemesterData, Event, Attendance, \
    Carpool, Uniform, PrivateEventMember, Repeat, GigRequest, AbsenceRequest


// @login_required
// @app.route('/events', methods=['GET'])
// id is optional
fn events(user: User) -> Result<Value, GreaseError> {
    json!(Event::load_all(user.conn)?.filter(|e| e.is_visible_to_user(user)).and_then(|e| {
        let all_attendance = Attendance::load_for_event(user.conn)?;
        json!({
            "event": e,
            "attendance": all_attendance.find(|a| a.member == user.member.email),
        })
    }))
}

// @login_required
// @app.route('/events/<int:event_id>', methods=['GET'])
pub fn single_event(user: User, event_id: i32) -> Result<Event, GreaseError> {
    let event = Event::load(event_id, user.conn)?;
    if !event.is_visible_to_member(user.member) {
        Err(GreaseError::forbidden())
    } else {
        Ok(event)
    }
}

// @login_required
// @app.route('/events/<int:event_id>/edit', methods=['GET'])
pub fn edit_event(user: User, event_id: i32) -> Result<impl Reply, GreaseError> {
    let event = Event::load(event_id, user.conn)?;
    abort_if_not_permitted_to('Edit Event:%s' % event.category.value)
    if !event.is_visible_to_user(user) || !user.can_for_event_type("Edit Event", event.type) {
        Err(GreaseError::forbidden())
    } else {
        Ok(json(json!({
            'event': event,
            'semesters': list(Semester.select().order_by(Semester.name)),
            'uniforms': list(Uniform.select().order_by(Uniform.name)),
            'active_members': list(User.select().join(
                UserSemesterData, on=(User.id == UserSemesterData.user)).where(
                    UserSemesterData.active == True).order_by(User.first_name, User.last_name))
        })))
    }
}
def edit_event(event_id):
    event = Event.get(Event.id == event_id)
    if not event.is_visible_to_user(current_user):
        abort(403)

    return render_template('events/edit.html.j2', **{
    })


// @login_required
// @app.route('/events/<int:event_id>/edit', methods=['POST'])
// requires a form!
pub fn update_event(user: User, event_id: i32, form: UpdatedEventForm) {
    // event = Event.get(Event.id == event_id)
    // abort_if_not_permitted_to('Edit Event:%s' % event.category.value)
    // if not event.is_visible_to_user(current_user):
    // abort(403)
    //
    // // TODO: try-catch for formatting errors
    // updates = {
    //     'title': data['title'].strip(),
    //     'location': data['location'].strip(),
    //     'category': EventCategory(data['category']),
    //     'start_time': datetime.strptime('%s %s' % (data['start_date'], data['start_time']),
    //     '%Y-%m-%d %H:%M'),
    //     'end_time': datetime.strptime('%s %s' % (data['end_date'], data['end_time']),
    //     '%Y-%m-%d %H:%M'),
    //     'semester': Semester.get(Semester.name == data['semester']).id,
    //     'point_multiplier': float(data['point_multiplier']),
    //     'description': data['description'].strip() or None,
    //     'required': bool(data.get('required')),
    //     'uniform': Uniform.get(Uniform.name == data['uniform']).id,
    //     'contact_name': data['contact_name'].strip() or None,
    //     'contact_email': data['contact_email'].strip() or None,
    //     'contact_phone': data['contact_phone'].strip() or None,
    //     'price': int(data['price']),
    //     'use_in_gig_count': bool(data.get('use_in_gig_count')),
    //     'public_description': data['public_description'].strip() or None,
    //     'is_private': bool(data.get('is_private'))
    // }
    //
    // try:
    // updates['performance_time'] = datetime.strptime(data['performance_time'], '%H:%M')
    // except ValueError:
    // pass
    //
    // if updates['is_private']:
    // old_member_ids = set([pem.user.id for pem in PrivateEventMember.select().where(
    //     PrivateEventMember.event == event_id)])
    //     PrivateEventMember.delete().where(PrivateEventMember.event == event_id)
    //     for user_email in data['allowed']:
    //     if data['allowed'][user_email]:
    //     PrivateEventMember.insert(event=event_id, user=User.get(
    //         User.email == user_email)).execute()
    //         current_member_ids = set([pem.user.id for pem in PrivateEventMember.select().where(
    //             PrivateEventMember.event == event_id)])
    //
    //             Attendance.delete().where(
    //                 Attendance.user.in_(old_member_ids.difference(current_member_ids))).execute()
    //                 Attendance.insert_many([{
    //                     'event': event_id,
    //                     'user': user_id,
    //                     'should_attend': updates['required']
    //                 } for user in current_member_ids.difference(old_member_ids)]).execute()
    //
    //                 Event.update(**updates).where(Event.id == event_id).execute()
    //                 return redirect('/events?id=%s' % event_id)
}


@permission_required('Is Officer')
@app.route('/events', methods=['POST'])
def submit_event():
    form = request.form.to_dict()

    data = {
        'title': form['title'].strip(),
        'location': form['location'].strip(),
        'category': EventCategory(form['category']),
        'start_time': datetime.strptime('%s %s' % (form['start_date'], form['start_time']),
                                        '%Y-%m-%d %H:%M'),
        'end_time': datetime.strptime('%s %s' % (form['end_date'], form['end_time']),
                                      '%Y-%m-%d %H:%M'),
        'semester': Semester.get(Semester.name == form['semester']).id,
        'description': form['description'].strip() or None,
        'required': bool(form.get('required')),
        'uniform': Uniform.get(Uniform.name == form['uniform']).id,
        'contact_name': form['contact_name'].strip() or None,
        'contact_email': form['contact_email'].strip() or None,
        'contact_phone': form['contact_phone'].strip() or None,
        'price': int(form['price']),
        'use_in_gig_count': bool(form.get('use_in_gig_count')),
        'public_description': form['public_description'].strip() or None,
        'is_private': bool(form.get('is_private'))
    }

    category = data['category'].value
    if data['is_private'] and not current_user.has_permission('Create Private Event:%s' % category):
        abort(403)
    elif not data['is_private']:
        abort_if_not_permitted_to('Create Event:%s' % category)

    try:
        data['performance_time'] = datetime.strptime(form['performance_time'], '%H:%M')
    except ValueError:
        pass

    created_events = []
    if form.get('repeat') != 'None':
        data['repeat'] = Repeat(form['repeat'])
        data['repeat_until'] = datetime.strptime(form['repeat_until'], '%Y-%m-%d').date()
        while data['start_time'].date() <= data['repeat_until']:
            created_events.append(Event.create(**data))
            Attendance.create_for_new_event(created_events[-1])
            data['start_time'] = data['repeat'].increment(data['start_time'])
            data['end_time'] = data['repeat'].increment(data['end_time'])
    else:
        data['repeat'], data['repeat_until'] = None, None
        created_events.append(Event.create(**data))
        Attendance.create_for_new_event(created_events[-1])

    if 'gig_request_id' in form:
        GigRequest.update(event=created_events[0].id).where(
            GigRequest.id == int(form['gig_request_id'])).execute()

    if data['is_private']:
        for event in created_events:
            for user_email in data['allowed']:
                if data['allowed'][user_email]:
                    PrivateEventMember.insert(event=event.id, user=User.get(
                        User.email == user_email)).execute()

    return redirect('/events?id=%d' % created_events[0].id)


@permission_required('Is Officer')
@app.route('/events/<int:event_id>/delete', methods=['POST'])
def delete_event(event_id):
    event = Event.get(Event.id == event_id)
    abort_if_not_permitted_to('Delete Event:%s' % event.category.value)
    if not event.is_visible_to_user(current_user):
        abort(403)

    event_title = event.title
    event.delete_instance()
    flash('Event "%s" has been successfully deleted.' % event_title)
    return redirect('/events')


@login_required
@app.route('/attendance/<int:event_id>', methods=['GET'])
def attendance(event_id):
    event = Event.get(Event.id == event_id)
    abort_if_not_permitted_to('Edit Attendance:%s' % event.category.value)

    return render_template('events/attendance.html.j2', **{
        'event': event,
        'section_attendances': Attendance.load_for_event_separate_by_section(event)
    })


@login_required
@app.route('/attendance/<int:attendance_id>', methods=['POST'])
def edit_attendance(attendance_id):
    attendance = Attendance.get_by_id(attendance_id)
    event_category = attendance.event.category
    abort_if_not_permitted_to('Edit Attendance:%s' % event_category.value)
    if not attendance.event.is_visible_to_user(current_user):
        abort(403)

    Attendance.update(**{k: v == 'true' if v in ['true', 'false'] else v
                         for k, v in request.form.to_dict().items()}).where(
        Attendance.id == attendance_id).execute()

    if request.args.get('return_grades', False):
        member = attendance.user
        event_grades, final_grade = member.calc_grades()
        return jsonify({
            'final_grade': final_grade,
            'event_grades': [{
                'event': event.to_json(),
                'reason': reason,
                'change': change,
                'attendance': Attendance.select().where(
                    (Attendance.user == member.id)
                    & (Attendance.event == event.id)).first().to_json()
            } for event, reason, change in event_grades]
        })

    else:
        return 'OK'


@login_required
@app.route('/events/<int:event_id>/carpools', methods=['GET'])
def carpools(event_id):
    event = Event.get_by_id(event_id)
    if not event.is_visible_to_user(current_user):
        abort(403)

    carpools = Carpool.load_for_event(event)
    return render_template('events/carpool.html.j2', **{
        'event': event,
        'carpools': carpools
    })


@permission_required('Edit Carpools')
@app.route('/events/<int:event_id>/edit_carpools', methods=['GET'])
def edit_carpools(event_id):
    event = Event.get_by_id(event_id)
    if not event.is_visible_to_user(current_user):
        abort(403)

    carpools = Carpool.load_for_event(event)
    members = [u for u in User.select().order_by(User.first_name, User.last_name)
               if u.active and u not in [c['carpool'].user for c in carpools]]
    if event.is_private:
        members = [u for u in members if u in event.allowed_members]

    return render_template('events/edit_carpool.html.j2', **{
        'event': event,
        'carpools': carpools,
        'members': members
    })


@permission_required('Edit Carpools')
@app.route('/events/<int:event_id>/update_carpools', methods=['POST'])
def update_carpools(event_id):
    if not Event.get(Event.id == event_id).is_visible_to_user(current_user):
        abort(403)

    Carpool.delete().where(Carpool.event == event_id).execute()
    for carpool in request.get_json(force=True):
        driver = User.get(User.email == carpool['driver'])
        Carpool.create(event=event_id, user=driver.id, is_driver=True, driver=None)

        passengers = [User.get(User.email == p_email) for p_email in carpool['passengers']]
        for passenger in passengers:
            Carpool.create(event=event_id, user=passenger.id, is_driver=False, driver=driver.id)

    return 'OK'


@permission_required('Process Gig Requests')
@app.route('/gig_requests', methods=['GET'])
def gig_requests():
    return render_template('events/gig_requests.html.j2', **{
        'gig_requests': GigRequest.select().order_by(GigRequest.time_submitted)
    })


@permission_required('Process Gig Requests')
@app.route('/gig_requests/<int:request_id>/decline', methods=['POST'])
def decline_gig_request(request_id):
    GigRequest.update(declined=True).where(GigRequest.id == request_id).execute()
    return 'OK'


@permission_required('Process Gig Requests')
@app.route('/gig_requests/<int:request_id>/reopen', methods=['POST'])
def reopen_gig_request(request_id):
    GigRequest.update(declined=False).where(GigRequest.id == request_id).execute()
    return 'OK'


@permission_required('Process Absence Requests')
@app.route('/absence_requests', methods=['GET'])
def absence_requests():
    return render_template('events/absence_requests.html.j2', **{
        'absence_requests': AbsenceRequest.select().join(
            Event, on=(AbsenceRequest.event == Event.id)).where(
            Event.semester == Semester.CURRENT.id).order_by(AbsenceRequest.time)
    })


@permission_required('Process Absence Requests')
@app.route('/absence_requests/<int:request_id>/update', methods=['POST'])
def update_absence_request(request_id):
    AbsenceRequest.update(**request.get_json(force=True)).where(
        AbsenceRequest.id == request_id).execute()
    return 'OK'
