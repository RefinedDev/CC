use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{LazyLock, Mutex};

use crate::routes::{
    auth::UserRecord,
    courses::{CourseRecord, LectureRecord},
};

static CONNECTION: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    let connection =
        Connection::open("capacity_connect.db").expect("failed to open capacity_connect.db");
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
         );
         CREATE TABLE IF NOT EXISTS assessments (
           id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, subject TEXT NOT NULL,
           deadline TEXT NOT NULL, duration_minutes INTEGER NOT NULL, created_by TEXT NOT NULL,
           FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS assessment_questions (
           id INTEGER PRIMARY KEY AUTOINCREMENT, assessment_id INTEGER NOT NULL,
           prompt TEXT NOT NULL, options_json TEXT NOT NULL, correct_option TEXT NOT NULL,
           position INTEGER NOT NULL,
           FOREIGN KEY(assessment_id) REFERENCES assessments(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS assessment_attempts (
           id INTEGER PRIMARY KEY AUTOINCREMENT, assessment_id INTEGER NOT NULL,
           user_id TEXT NOT NULL, score INTEGER NOT NULL, total INTEGER NOT NULL,
           submitted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           UNIQUE(assessment_id, user_id),
           FOREIGN KEY(assessment_id) REFERENCES assessments(id) ON DELETE CASCADE,
           FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS resources (
           id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, kind TEXT NOT NULL,
           size_bytes INTEGER NOT NULL, course_id TEXT, created_by TEXT NOT NULL,
           content_base64 TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           FOREIGN KEY(course_id) REFERENCES courses(id) ON DELETE SET NULL,
           FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS publications (
           id INTEGER PRIMARY KEY AUTOINCREMENT, kind TEXT NOT NULL, title TEXT NOT NULL,
           body TEXT NOT NULL, created_by TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE CASCADE
         );
         CREATE TABLE IF NOT EXISTS notification_reads (
           publication_id INTEGER NOT NULL, user_id TEXT NOT NULL, read_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (publication_id, user_id),
           FOREIGN KEY(publication_id) REFERENCES publications(id) ON DELETE CASCADE,
           FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
         );",
    )?;
    let has_target: bool = connection.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('publications') WHERE name='target_user_id'",
        [],
        |row| row.get::<_, i64>(0),
    )? > 0;
    if !has_target {
        connection.execute("ALTER TABLE publications ADD COLUMN target_user_id TEXT REFERENCES users(id) ON DELETE CASCADE", [])?;
    }
    connection.execute(
        "INSERT INTO lectures (course_id, title, description, position)
         SELECT c.id, c.title || ' Overview', c.description, 1
         FROM courses c
         WHERE NOT EXISTS (SELECT 1 FROM lectures l WHERE l.course_id = c.id)",
        [],
    )?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PublicationRecord {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub target_user_id: Option<String>,
    pub read: bool,
}

pub fn list_publications(kind: Option<&str>, user_id: &str) -> rusqlite::Result<Vec<PublicationRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT p.id,p.kind,p.title,p.body,p.created_at,p.target_user_id,EXISTS(SELECT 1 FROM notification_reads r WHERE r.publication_id=p.id AND r.user_id=?2) FROM publications p WHERE (?1 IS NULL OR p.kind=?1) AND (p.target_user_id IS NULL OR p.target_user_id=?2) ORDER BY p.created_at DESC,p.id DESC")?;
    let rows = statement.query_map(params![kind, user_id], |row| {
        Ok(PublicationRecord {
            id: row.get(0)?,
            kind: row.get(1)?,
            title: row.get(2)?,
            body: row.get(3)?,
            created_at: row.get(4)?,
            target_user_id: row.get(5)?,
            read: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn insert_publication(
    kind: &str,
    title: &str,
    body: &str,
    created_by: &str,
    target_user_id: Option<&str>,
) -> rusqlite::Result<PublicationRecord> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "INSERT INTO publications (kind,title,body,created_by,target_user_id) VALUES (?1,?2,?3,?4,?5)",
        params![kind, title, body, created_by, target_user_id],
    )?;
    let id = connection.last_insert_rowid();
    connection.query_row(
        "SELECT id,kind,title,body,created_at,target_user_id FROM publications WHERE id=?1",
        params![id],
        |row| {
            Ok(PublicationRecord {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                body: row.get(3)?,
                created_at: row.get(4)?,
                target_user_id: row.get(5)?,
                read: false,
            })
        },
    )
}

pub fn mark_publication_read(publication_id: i64, user_id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute(
        "INSERT OR IGNORE INTO notification_reads (publication_id,user_id) SELECT id,?2 FROM publications WHERE id=?1 AND (target_user_id IS NULL OR target_user_id=?2)",
        params![publication_id, user_id],
    )? > 0)
}

pub fn mark_all_publications_read(user_id: &str) -> rusqlite::Result<usize> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "INSERT OR IGNORE INTO notification_reads (publication_id,user_id) SELECT id,?1 FROM publications WHERE target_user_id IS NULL OR target_user_id=?1",
        params![user_id],
    )
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceRecord {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub size_bytes: i64,
    pub course_id: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

pub fn list_resources() -> rusqlite::Result<Vec<ResourceRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT id,name,kind,size_bytes,course_id,created_by,created_at FROM resources ORDER BY created_at DESC,id DESC")?;
    let rows = statement.query_map([], |row| {
        Ok(ResourceRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: row.get(2)?,
            size_bytes: row.get(3)?,
            course_id: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn insert_resource(
    name: &str,
    kind: &str,
    size_bytes: i64,
    course_id: Option<&str>,
    created_by: &str,
    content_base64: &str,
) -> rusqlite::Result<i64> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute("INSERT INTO resources (name,kind,size_bytes,course_id,created_by,content_base64) VALUES (?1,?2,?3,?4,?5,?6)",
        params![name, kind, size_bytes, course_id, created_by, content_base64])?;
    Ok(connection.last_insert_rowid())
}

pub fn resource_content(id: i64) -> rusqlite::Result<Option<(ResourceRecord, String)>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row("SELECT id,name,kind,size_bytes,course_id,created_by,created_at,content_base64 FROM resources WHERE id=?1", params![id], |row| Ok((
        ResourceRecord { id: row.get(0)?, name: row.get(1)?, kind: row.get(2)?, size_bytes: row.get(3)?, course_id: row.get(4)?, created_by: row.get(5)?, created_at: row.get(6)? },
        row.get(7)?,
    ))).optional()
}

pub fn delete_resource(id: i64, user_id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute(
        "DELETE FROM resources WHERE id=?1 AND created_by=?2",
        params![id, user_id],
    )? > 0)
}

pub fn course_analytics(user_id: &str) -> rusqlite::Result<Vec<serde_json::Value>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT c.id,c.title,COUNT(DISTINCT e.user_id),
                COUNT(DISTINCT l.id),
                COUNT(DISTINCT CASE WHEN p.completed=1 THEN p.user_id || ':' || p.lecture_id END)
         FROM courses c
         LEFT JOIN enrollments e ON e.course_id=c.id
         LEFT JOIN lectures l ON l.course_id=c.id
         LEFT JOIN lecture_progress p ON p.lecture_id=l.id AND p.user_id=e.user_id
         WHERE c.created_by=?1 GROUP BY c.id,c.title ORDER BY c.title",
    )?;
    let rows = statement.query_map(params![user_id], |row| {
        let enrollments: i64 = row.get(2)?;
        let lectures: i64 = row.get(3)?;
        let completed: i64 = row.get(4)?;
        let possible = enrollments * lectures;
        Ok(serde_json::json!({
            "course_id": row.get::<_, String>(0)?, "title": row.get::<_, String>(1)?,
            "enrollments": enrollments, "lectures": lectures, "completed_lectures": completed,
            "completion_rate": if possible == 0 { 0.0 } else { (completed as f64 / possible as f64 * 100.0).round() }
        }))
    })?;
    rows.collect()
}

pub fn assessment_analytics(user_id: &str) -> rusqlite::Result<Vec<serde_json::Value>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT a.id,a.title,a.subject,COUNT(at.id),COALESCE(AVG(CASE WHEN at.total=0 THEN 0.0 ELSE at.score*100.0/at.total END),0)
         FROM assessments a LEFT JOIN assessment_attempts at ON at.assessment_id=a.id
         WHERE a.created_by=?1 GROUP BY a.id,a.title,a.subject ORDER BY a.id DESC")?;
    let rows = statement.query_map(params![user_id], |row| {
        Ok(serde_json::json!({
            "assessment_id": row.get::<_, i64>(0)?, "title": row.get::<_, String>(1)?,
            "subject": row.get::<_, String>(2)?, "attempts": row.get::<_, i64>(3)?,
            "average_score": row.get::<_, f64>(4)?.round()
        }))
    })?;
    rows.collect()
}

pub fn trainee_progress_analytics(user_id: &str) -> rusqlite::Result<Vec<serde_json::Value>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT u.id,u.name,c.id,c.title,COUNT(DISTINCT l.id),
                COUNT(DISTINCT CASE WHEN p.completed=1 THEN p.lecture_id END)
         FROM users u
         JOIN enrollments e ON e.user_id=u.id
         JOIN courses c ON c.id=e.course_id
         LEFT JOIN lectures l ON l.course_id=c.id
         LEFT JOIN lecture_progress p ON p.lecture_id=l.id AND p.user_id=u.id
         WHERE c.created_by=?1
         GROUP BY u.id,u.name,c.id,c.title ORDER BY u.name,c.title",
    )?;
    let rows = statement.query_map(params![user_id], |row| {
        let total: i64 = row.get(4)?;
        let completed: i64 = row.get(5)?;
        Ok(serde_json::json!({
            "trainee_id": row.get::<_, String>(0)?, "trainee_name": row.get::<_, String>(1)?,
            "course_id": row.get::<_, String>(2)?, "course_title": row.get::<_, String>(3)?,
            "completed": completed, "total": total,
            "completion_rate": if total == 0 { 0 } else { (completed * 100 / total) }
        }))
    })?;
    rows.collect()
}

pub fn assessment_attempt_details(user_id: &str) -> rusqlite::Result<Vec<serde_json::Value>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT a.id,a.title,u.name,at.score,at.total,at.submitted_at
         FROM assessment_attempts at
         JOIN assessments a ON a.id=at.assessment_id
         JOIN users u ON u.id=at.user_id
         WHERE a.created_by=?1 ORDER BY at.submitted_at DESC",
    )?;
    let rows = statement.query_map(params![user_id], |row| {
        let score: i64 = row.get(3)?;
        let total: i64 = row.get(4)?;
        Ok(serde_json::json!({
            "assessment_id": row.get::<_, i64>(0)?, "assessment_title": row.get::<_, String>(1)?,
            "trainee_name": row.get::<_, String>(2)?, "score": score, "total": total,
            "percent": if total == 0 { 0 } else { score * 100 / total },
            "submitted_at": row.get::<_, String>(5)?
        }))
    })?;
    rows.collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssessmentRecord {
    pub id: i64,
    pub title: String,
    pub subject: String,
    pub deadline: String,
    pub duration_minutes: i64,
    pub created_by: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssessmentQuestionRecord {
    pub id: i64,
    pub assessment_id: i64,
    pub prompt: String,
    pub options: Vec<String>,
    pub position: i64,
}

pub fn list_assessments() -> rusqlite::Result<Vec<AssessmentRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT id,title,subject,deadline,duration_minutes,created_by FROM assessments ORDER BY deadline,id")?;
    let rows = statement.query_map([], |row| {
        Ok(AssessmentRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            subject: row.get(2)?,
            deadline: row.get(3)?,
            duration_minutes: row.get(4)?,
            created_by: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn find_assessment(id: i64) -> rusqlite::Result<Option<AssessmentRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row("SELECT id,title,subject,deadline,duration_minutes,created_by FROM assessments WHERE id=?1", params![id], |row| Ok(AssessmentRecord {
        id: row.get(0)?, title: row.get(1)?, subject: row.get(2)?, deadline: row.get(3)?,
        duration_minutes: row.get(4)?, created_by: row.get(5)?,
    })).optional()
}

pub fn delete_assessment(id: i64, user_id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute(
        "DELETE FROM assessments WHERE id = ?1 AND created_by = ?2",
        params![id, user_id],
    )? > 0)
}

pub fn assessment_questions(
    id: i64,
    include_answers: bool,
) -> rusqlite::Result<Vec<(AssessmentQuestionRecord, String)>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare("SELECT id,assessment_id,prompt,options_json,correct_option,position FROM assessment_questions WHERE assessment_id=?1 ORDER BY position,id")?;
    let rows = statement.query_map(params![id], |row| {
        let options_json: String = row.get(3)?;
        let options = serde_json::from_str(&options_json).unwrap_or_default();
        let correct: String = row.get(4)?;
        Ok((
            AssessmentQuestionRecord {
                id: row.get(0)?,
                assessment_id: row.get(1)?,
                prompt: row.get(2)?,
                options,
                position: row.get(5)?,
            },
            if include_answers {
                correct
            } else {
                String::new()
            },
        ))
    })?;
    rows.collect()
}

pub fn insert_assessment(
    assessment: &AssessmentRecord,
    questions: &[(String, Vec<String>, String)],
) -> rusqlite::Result<()> {
    let mut connection = CONNECTION.lock().unwrap();
    let transaction = connection.transaction()?;
    transaction.execute("INSERT INTO assessments (title,subject,deadline,duration_minutes,created_by) VALUES (?1,?2,?3,?4,?5)",
        params![assessment.title, assessment.subject, assessment.deadline, assessment.duration_minutes, assessment.created_by])?;
    let id = transaction.last_insert_rowid();
    for (position, (prompt, options, correct)) in questions.iter().enumerate() {
        transaction.execute("INSERT INTO assessment_questions (assessment_id,prompt,options_json,correct_option,position) VALUES (?1,?2,?3,?4,?5)",
            params![id, prompt, serde_json::to_string(options).unwrap(), correct, position as i64 + 1])?;
    }
    transaction.commit()
}

pub fn assessment_result(
    assessment_id: i64,
    user_id: &str,
) -> rusqlite::Result<Option<(i64, i64, String)>> {
    let connection = CONNECTION.lock().unwrap();
    connection.query_row("SELECT score,total,submitted_at FROM assessment_attempts WHERE assessment_id=?1 AND user_id=?2", params![assessment_id, user_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()
}

pub fn submit_assessment(
    assessment_id: i64,
    user_id: &str,
    answers: &std::collections::HashMap<String, String>,
) -> rusqlite::Result<Option<(i64, i64, String)>> {
    let mut connection = CONNECTION.lock().unwrap();
    let transaction = connection.transaction()?;
    let questions = {
        let mut statement = transaction.prepare("SELECT id,correct_option FROM assessment_questions WHERE assessment_id=?1 ORDER BY position,id")?;
        let rows = statement.query_map(params![assessment_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    if questions.is_empty()
        || transaction
            .query_row(
                "SELECT id FROM assessments WHERE id=?1",
                params![assessment_id],
                |_| Ok(()),
            )
            .optional()?
            .is_none()
    {
        return Ok(None);
    }
    let score = questions
        .iter()
        .filter(|(id, correct)| answers.get(&id.to_string()) == Some(correct))
        .count() as i64;
    let total = questions.len() as i64;
    transaction.execute("INSERT INTO assessment_attempts (assessment_id,user_id,score,total) VALUES (?1,?2,?3,?4) ON CONFLICT(assessment_id,user_id) DO UPDATE SET score=excluded.score,total=excluded.total,submitted_at=CURRENT_TIMESTAMP",
        params![assessment_id, user_id, score, total])?;
    let result = transaction.query_row("SELECT score,total,submitted_at FROM assessment_attempts WHERE assessment_id=?1 AND user_id=?2", params![assessment_id, user_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    transaction.commit()?;
    Ok(Some(result))
}

pub fn find_user_by_id(id: &str) -> rusqlite::Result<Option<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection
        .query_row(
            "SELECT id, name, email, password_hash, role FROM users WHERE id = ?1",
            params![id],
            |row| {
                Ok(UserRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: row.get(4)?,
                })
            },
        )
        .optional()
}

pub fn find_user_by_email(email: &str) -> rusqlite::Result<Option<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection
        .query_row(
            "SELECT id, name, email, password_hash, role FROM users WHERE email = ?1",
            params![email],
            |row| {
                Ok(UserRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: row.get(4)?,
                })
            },
        )
        .optional()
}

pub fn list_users() -> rusqlite::Result<Vec<UserRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection
        .prepare("SELECT id, name, email, password_hash, role FROM users ORDER BY name")?;
    let rows = statement.query_map([], |row| {
        Ok(UserRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            password_hash: row.get(3)?,
            role: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn dashboard_counts() -> rusqlite::Result<(i64, i64, i64, i64, i64, i64)> {
    let connection = CONNECTION.lock().unwrap();
    let users: i64 = connection.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    let courses: i64 =
        connection.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;
    let enrollments: i64 =
        connection.query_row("SELECT COUNT(*) FROM enrollments", [], |row| row.get(0))?;
    let trainees: i64 = connection.query_row(
        "SELECT COUNT(*) FROM users WHERE lower(role) = 'trainee'",
        [],
        |row| row.get(0),
    )?;
    let trainers: i64 = connection.query_row(
        "SELECT COUNT(*) FROM users WHERE lower(role) = 'trainer'",
        [],
        |row| row.get(0),
    )?;
    let admins: i64 = connection.query_row(
        "SELECT COUNT(*) FROM users WHERE lower(role) = 'admin'",
        [],
        |row| row.get(0),
    )?;
    Ok((users, courses, enrollments, trainees, trainers, admins))
}

pub fn insert_user(user: &UserRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "INSERT INTO users (id, name, email, password_hash, role) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            user.id,
            user.name,
            user.email,
            user.password_hash,
            user.role
        ],
    )?;
    Ok(())
}

pub fn update_user(user: &UserRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "UPDATE users SET name = ?1, email = ?2, role = ?3 WHERE id = ?4",
        params![user.name, user.email, user.role, user.id],
    )?;
    Ok(())
}

pub fn delete_user(id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute("DELETE FROM users WHERE id = ?1", params![id])? > 0)
}

pub fn list_courses() -> rusqlite::Result<Vec<CourseRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement =
        connection.prepare("SELECT id, title, description, created_by FROM courses ORDER BY id")?;
    let rows = statement.query_map([], |row| {
        Ok(CourseRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            created_by: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn find_course(id: &str) -> rusqlite::Result<Option<CourseRecord>> {
    let connection = CONNECTION.lock().unwrap();
    connection
        .query_row(
            "SELECT id, title, description, created_by FROM courses WHERE id = ?1",
            params![id],
            |row| {
                Ok(CourseRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_by: row.get(3)?,
                })
            },
        )
        .optional()
}

pub fn next_course_id() -> rusqlite::Result<String> {
    let connection = CONNECTION.lock().unwrap();
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;
    Ok(format!("course_{}", count + 1))
}

pub fn insert_course(course: &CourseRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "INSERT INTO courses (id, title, description, created_by) VALUES (?1, ?2, ?3, ?4)",
        params![
            course.id,
            course.title,
            course.description,
            course.created_by
        ],
    )?;
    connection.execute(
        "INSERT INTO lectures (course_id, title, description, position) VALUES (?1, ?2, ?3, 1)",
        params![
            course.id,
            format!("{} Overview", course.title),
            course.description
        ],
    )?;
    Ok(())
}

pub fn update_course(course: &CourseRecord) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "UPDATE courses SET title = ?1, description = ?2 WHERE id = ?3",
        params![course.title, course.description, course.id],
    )?;
    Ok(())
}

pub fn delete_course(id: &str) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    Ok(connection.execute("DELETE FROM courses WHERE id = ?1", params![id])? > 0)
}

pub fn enrolled_users(course_id: &str) -> rusqlite::Result<Vec<String>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection
        .prepare("SELECT user_id FROM enrollments WHERE course_id = ?1 ORDER BY user_id")?;
    let rows = statement.query_map(params![course_id], |row| row.get(0))?;
    rows.collect()
}

pub fn enroll(course_id: &str, user_id: &str) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "INSERT OR IGNORE INTO enrollments (course_id, user_id) VALUES (?1, ?2)",
        params![course_id, user_id],
    )?;
    Ok(())
}

pub fn unenroll(course_id: &str, user_id: &str) -> rusqlite::Result<()> {
    let connection = CONNECTION.lock().unwrap();
    connection.execute(
        "DELETE FROM enrollments WHERE course_id = ?1 AND user_id = ?2",
        params![course_id, user_id],
    )?;
    Ok(())
}

pub fn list_lectures(course_id: &str) -> rusqlite::Result<Vec<LectureRecord>> {
    let connection = CONNECTION.lock().unwrap();
    let mut statement = connection.prepare(
        "SELECT id, course_id, title, description, position FROM lectures WHERE course_id = ?1 ORDER BY position, id",
    )?;
    let rows = statement.query_map(params![course_id], |row| {
        Ok(LectureRecord {
            id: row.get(0)?,
            course_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            position: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn insert_lecture(
    course_id: &str,
    title: &str,
    description: &str,
) -> rusqlite::Result<LectureRecord> {
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
    Ok(LectureRecord {
        id,
        course_id: course_id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        position,
    })
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

pub fn mark_lecture_complete(
    user_id: &str,
    course_id: &str,
    lecture_id: i64,
) -> rusqlite::Result<bool> {
    let connection = CONNECTION.lock().unwrap();
    let exists: Option<i64> = connection
        .query_row(
            "SELECT id FROM lectures WHERE id = ?1 AND course_id = ?2",
            params![lecture_id, course_id],
            |row| row.get(0),
        )
        .optional()?;
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
