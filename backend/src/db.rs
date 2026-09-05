use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{LazyLock, Mutex};

use crate::routes::{auth::UserRecord, courses::{CourseRecord, LectureRecord}};

static CONNECTION: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    let connection = Connection::open("capacity_connect.db")
        .expect("failed to open capacity_connect.db");
    Mutex::new(connection)
});

pub fn init() -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS users (
           id TEXT PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL UNIQUE,
           password_hash TEXT NOT NULL, role TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS courses (
           id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT NOT NULL,
           created_by TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS enrollments (
           course_id TEXT NOT NULL, user_id TEXT NOT NULL,
           PRIMARY KEY (course_id, user_id),
           FOREIGN KEY(course_id) REFERENCES courses(id) ON DELETE CASCADE,
           FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS lectures (
           id INTEGER PRIMARY KEY AUTOINCREMENT, course_id TEXT NOT NULL,
           title TEXT NOT NULL, description TEXT NOT NULL, position INTEGER NOT NULL,
           FOREIGN KEY(course_id) REFERENCES courses(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS lecture_progress (
           user_id TEXT NOT NULL, lecture_id INTEGER NOT NULL, completed INTEGER NOT NULL DEFAULT 0,
           PRIMARY KEY (user_id, lecture_id),
           FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
           FOREIGN KEY(lecture_id) REFERENCES lectures(id) ON DELETE CASCADE
         );",
    )?;
    connection.execute(
        "INSERT INTO lectures (course_id, title, description, position)
         SELECT c.id, c.title || ' Overview', c.description, 1
         FROM courses c
         WHERE NOT EXISTS (SELECT 1 FROM lectures l WHERE l.course_id = c.id)",
        [],
    )?;
    Ok(())
}

pub fn find_user_by_id(id: &str) -> rusqlite::Result<Option<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row(
        "SELECT id, name, email, password_hash, role FROM users WHERE id = ?1",
        params![id],
        |row| Ok(UserRecord { id: row.get(0)?, name: row.get(1)?, email: row.get(2)?, password_hash: row.get(3)?, role: row.get(4)? }),
    ).optional()
}

pub fn find_user_by_email(email: &str) -> rusqlite::Result<Option<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row(
        "SELECT id, name, email, password_hash, role FROM users WHERE email = ?1",
        params![email],
        |row| Ok(UserRecord { id: row.get(0)?, name: row.get(1)?, email: row.get(2)?, password_hash: row.get(3)?, role: row.get(4)? }),
    ).optional()
}

pub fn list_users() -> rusqlite::Result<Vec<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT id, name, email, password_hash, role FROM users ORDER BY name",
    )?;
    let rows = statement.query_map([], |row| Ok(UserRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
        password_hash: row.get(3)?,
        role: row.get(4)?,
    }))?;
    rows.collect()
}

pub fn dashboard_counts() -> rusqlite::Result<(i64, i64, i64, i64, i64, i64)> {
    let connection = CONNECTION.lock().unwrap();
    let users: i64 = connection.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    let courses: i64 = connection.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;
    let enrollments: i64 = connection.query_row("SELECT COUNT(*) FROM enrollments", [], |row| row.get(0))?;
    let trainees: i64 = connection.query_row("SELECT COUNT(*) FROM users WHERE lower(role) = 'trainee'", [], |row| row.get(0))?;
    let trainers: i64 = connection.query_row("SELECT COUNT(*) FROM users WHERE lower(role) = 'trainer'", [], |row| row.get(0))?;
    let admins: i64 = connection.query_row("SELECT COUNT(*) FROM users WHERE lower(role) = 'admin'", [], |row| row.get(0))?;
    Ok((users, courses, enrollments, trainees, trainers, admins))
}

pub fn insert_user(user: &UserRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("INSERT INTO users (id, name, email, password_hash, role) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user.id, user.name, user.email, user.password_hash, user.role])?;
    Ok(())
}

pub fn update_user(user: &UserRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("UPDATE users SET name = ?1, email = ?2, role = ?3 WHERE id = ?4",
        params![user.name, user.email, user.role, user.id])?;
    Ok(())
}

pub fn delete_user(id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute("DELETE FROM users WHERE id = ?1", params![id])? > 0)
}

pub fn list_courses() -> rusqlite::Result<Vec<CourseRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT id, title, description, created_by FROM courses ORDER BY id")?;
    let rows = statement.query_map([], |row| Ok(CourseRecord { id: row.get(0)?, title: row.get(1)?, description: row.get(2)?, created_by: row.get(3)? }))?;
    rows.collect()
}

pub fn find_course(id: &str) -> rusqlite::Result<Option<CourseRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row("SELECT id, title, description, created_by FROM courses WHERE id = ?1", params![id],
        |row| Ok(CourseRecord { id: row.get(0)?, title: row.get(1)?, description: row.get(2)?, created_by: row.get(3)? })).optional()
}

pub fn next_course_id() -> rusqlite::Result<String> {
    let connection = CONNECTION.lock().unwrap();
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;
    Ok(format!("course_{}", count + 1))
}

pub fn insert_course(course: &CourseRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("INSERT INTO courses (id, title, description, created_by) VALUES (?1, ?2, ?3, ?4)",
        params![course.id, course.title, course.description, course.created_by])?;
    connection.execute("INSERT INTO lectures (course_id, title, description, position) VALUES (?1, ?2, ?3, 1)",
        params![course.id, format!("{} Overview", course.title), course.description])?;
    Ok(())
}

pub fn update_course(course: &CourseRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("UPDATE courses SET title = ?1, description = ?2 WHERE id = ?3",
        params![course.title, course.description, course.id])?;
    Ok(())
}

pub fn delete_course(id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute("DELETE FROM courses WHERE id = ?1", params![id])? > 0)
}

pub fn enrolled_users(course_id: &str) -> rusqlite::Result<Vec<String>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT user_id FROM enrollments WHERE course_id = ?1 ORDER BY user_id")?;
    let rows = statement.query_map(params![course_id], |row| row.get(0))?;
    rows.collect()
}

pub fn enroll(course_id: &str, user_id: &str) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("INSERT OR IGNORE INTO enrollments (course_id, user_id) VALUES (?1, ?2)", params![course_id, user_id])?;
    Ok(())
}

pub fn unenroll(course_id: &str, user_id: &str) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("DELETE FROM enrollments WHERE course_id = ?1 AND user_id = ?2", params![course_id, user_id])?;
    Ok(())
}

pub fn list_lectures(course_id: &str) -> rusqlite::Result<Vec<LectureRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT id, course_id, title, description, position FROM lectures WHERE course_id = ?1 ORDER BY position, id",
    )?;
    let rows = statement.query_map(params![course_id], |row| Ok(LectureRecord {
        id: row.get(0)?,
        course_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        position: row.get(4)?,
    }))?;
    rows.collect()
}

pub fn insert_lecture(course_id: &str, title: &str, description: &str) -> rusqlite::Result<LectureRecord> {
    let connection = CONNECTION.lock().unwrap();
    let position: i64 = connection.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM lectures WHERE course_id = ?1",
        params![course_id],
        |row| row.get(0),
    )?;
    connection.execute(
        "INSERT INTO lectures (course_id, title, description, position) VALUES (?1, ?2, ?3, ?4)",
        params![course_id, title, description, position],
    )?;
    let id = connection.last_insert_rowid();
    Ok(LectureRecord { id, course_id: course_id.to_string(), title: title.to_string(), description: description.to_string(), position })
}

pub fn delete_lecture(course_id: &str, lecture_id: i64) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute(
        "DELETE FROM lectures WHERE id = ?1 AND course_id = ?2",
        params![lecture_id, course_id],
    )? > 0)
}

pub fn completed_lectures(user_id: &str, course_id: &str) -> rusqlite::Result<Vec<i64>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT p.lecture_id FROM lecture_progress p
         JOIN lectures l ON l.id = p.lecture_id
         WHERE p.user_id = ?1 AND l.course_id = ?2 AND p.completed = 1
         ORDER BY l.position, l.id",
    )?;
    let rows = statement.query_map(params![user_id, course_id], |row| row.get(0))?;
    rows.collect()
}

pub fn mark_lecture_complete(user_id: &str, course_id: &str, lecture_id: i64) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    let exists: Option<i64> = connection.query_row(
        "SELECT id FROM lectures WHERE id = ?1 AND course_id = ?2",
        params![lecture_id, course_id],
        |row| row.get(0),
    ).optional()?;
    if exists.is_none() {
        return Ok(false);
    }
    connection.execute(
        "INSERT INTO lecture_progress (user_id, lecture_id, completed)
         VALUES (?1, ?2, 1)
         ON CONFLICT(user_id, lecture_id) DO UPDATE SET completed = 1",
        params![user_id, lecture_id],
    )?;
    Ok(true)
}
