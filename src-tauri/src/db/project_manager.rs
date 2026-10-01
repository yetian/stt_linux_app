use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{NewRecording, Project, Recording, Tag};

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS recordings (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    file_name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    audio_duration_secs REAL,
    detected_language TEXT,
    status TEXT CHECK(status IN ('pending', 'transcribing', 'summarizing', 'completed', 'failed')),
    transcript_raw TEXT,
    summary_markdown TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recording_id TEXT,
    tag_name TEXT NOT NULL,
    FOREIGN KEY(recording_id) REFERENCES recordings(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recordings_project ON recordings(project_id);
CREATE INDEX IF NOT EXISTS idx_recordings_status ON recordings(status);
CREATE INDEX IF NOT EXISTS idx_tags_recording ON tags(recording_id);
"#;

const PROJECT_COLUMNS: &str = "p.id, p.name, p.description, p.created_at, \
     (SELECT COUNT(*) FROM recordings r WHERE r.project_id = p.id)";

const RECORDING_COLUMNS: &str = "r.id, r.project_id, r.file_name, r.source_path, \
     r.audio_duration_secs, r.detected_language, r.status, r.transcript_raw, \
     r.summary_markdown, r.created_at";

pub fn init_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        recording_count: row.get(4)?,
    })
}

fn recording_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Recording> {
    Ok(Recording {
        id: row.get(0)?,
        project_id: row.get(1)?,
        file_name: row.get(2)?,
        source_path: row.get(3)?,
        audio_duration_secs: row.get(4)?,
        detected_language: row.get(5)?,
        status: row.get(6)?,
        transcript_raw: row.get(7)?,
        summary_markdown: row.get(8)?,
        created_at: row.get(9)?,
    })
}

pub fn create_project(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
) -> AppResult<Project> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("project name must not be empty"));
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO projects (id, name, description) VALUES (?1, ?2, ?3)",
        params![id, trimmed, description],
    )?;

    get_project(conn, &id)
}

pub fn get_project(conn: &Connection, id: &str) -> AppResult<Project> {
    let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects p WHERE p.id = ?1");
    conn.query_row(&sql, params![id], project_from_row)
        .map_err(Into::into)
}

pub fn list_projects(conn: &Connection) -> AppResult<Vec<Project>> {
    let sql = format!(
        "SELECT {PROJECT_COLUMNS} FROM projects p ORDER BY p.created_at ASC, p.name ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], project_from_row)?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

pub fn delete_project(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn add_recording(conn: &Connection, input: &NewRecording) -> AppResult<Recording> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO recordings (id, project_id, file_name, source_path, audio_duration_secs, status) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
        params![
            id,
            input.project_id,
            input.file_name,
            input.source_path,
            input.audio_duration_secs
        ],
    )?;

    get_recording(conn, &id)
}

pub fn get_recording(conn: &Connection, id: &str) -> AppResult<Recording> {
    let sql = format!("SELECT {RECORDING_COLUMNS} FROM recordings r WHERE r.id = ?1");
    conn.query_row(&sql, params![id], recording_from_row)
        .map_err(Into::into)
}

pub fn list_recordings(conn: &Connection, project_id: Option<&str>) -> AppResult<Vec<Recording>> {
    match project_id {
        Some(pid) => {
            let sql = format!(
                "SELECT {RECORDING_COLUMNS} FROM recordings r WHERE r.project_id = ?1 ORDER BY r.created_at DESC"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![pid], recording_from_row)?;
            collect(rows)
        }
        None => {
            let sql = format!(
                "SELECT {RECORDING_COLUMNS} FROM recordings r ORDER BY r.created_at DESC"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], recording_from_row)?;
            collect(rows)
        }
    }
}

pub fn search_recordings(
    conn: &Connection,
    keyword: Option<&str>,
    project_id: Option<&str>,
    tag_filter: Option<&str>,
) -> AppResult<Vec<Recording>> {
    let mut sql = format!("SELECT {RECORDING_COLUMNS} FROM recordings r WHERE 1 = 1");
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(pid) = project_id {
        sql.push_str(" AND r.project_id = ?");
        args.push(Box::new(pid.to_string()));
    }

    if let Some(keyword) = keyword {
        let trimmed = keyword.trim();
        if !trimmed.is_empty() {
            let like = format!("%{trimmed}%");
            sql.push_str(
                " AND (r.file_name LIKE ? OR r.transcript_raw LIKE ? OR r.summary_markdown LIKE ?)",
            );
            args.push(Box::new(like.clone()));
            args.push(Box::new(like.clone()));
            args.push(Box::new(like));
        }
    }

    if let Some(tag) = tag_filter {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM tags t WHERE t.recording_id = r.id AND t.tag_name = ?)",
            );
            args.push(Box::new(trimmed.to_string()));
        }
    }

    sql.push_str(" ORDER BY r.created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|arg| arg.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), recording_from_row)?;
    collect(rows)
}

pub fn assign_recording_to_project(
    conn: &Connection,
    recording_id: &str,
    project_id: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE recordings SET project_id = ?1 WHERE id = ?2",
        params![project_id, recording_id],
    )?;
    Ok(())
}

pub fn update_recording_status(
    conn: &Connection,
    recording_id: &str,
    status: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE recordings SET status = ?1 WHERE id = ?2",
        params![status, recording_id],
    )?;
    Ok(())
}

pub fn update_recording_metadata(
    conn: &Connection,
    recording_id: &str,
    duration_secs: Option<f64>,
    detected_language: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE recordings SET audio_duration_secs = COALESCE(?1, audio_duration_secs), \
         detected_language = COALESCE(?2, detected_language) WHERE id = ?3",
        params![duration_secs, detected_language, recording_id],
    )?;
    Ok(())
}

pub fn save_transcript(conn: &Connection, recording_id: &str, transcript: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE recordings SET transcript_raw = ?1 WHERE id = ?2",
        params![transcript, recording_id],
    )?;
    Ok(())
}

pub fn save_summary(conn: &Connection, recording_id: &str, summary: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE recordings SET summary_markdown = ?1 WHERE id = ?2",
        params![summary, recording_id],
    )?;
    Ok(())
}

pub fn delete_recording(conn: &Connection, recording_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM recordings WHERE id = ?1", params![recording_id])?;
    Ok(())
}

pub fn add_tag(conn: &Connection, recording_id: &str, tag_name: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO tags (recording_id, tag_name) VALUES (?1, ?2)",
        params![recording_id, tag_name],
    )?;
    Ok(())
}

pub fn list_tags(conn: &Connection, recording_id: &str) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, recording_id, tag_name FROM tags WHERE recording_id = ?1 ORDER BY tag_name ASC",
    )?;
    let rows = stmt.query_map(params![recording_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            recording_id: row.get(1)?,
            tag_name: row.get(2)?,
        })
    })?;

    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn rename_project(conn: &Connection, id: &str, name: &str) -> AppResult<Project> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("project name must not be empty"));
    }

    conn.execute(
        "UPDATE projects SET name = ?1 WHERE id = ?2",
        params![trimmed, id],
    )?;

    get_project(conn, id)
}

pub fn rename_recording(conn: &Connection, id: &str, file_name: &str) -> AppResult<Recording> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("recording name must not be empty"));
    }

    conn.execute(
        "UPDATE recordings SET file_name = ?1 WHERE id = ?2",
        params![trimmed, id],
    )?;

    get_recording(conn, id)
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
    Ok(())
}

fn collect(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Recording>>,
) -> AppResult<Vec<Recording>> {
    let mut recordings = Vec::new();
    for row in rows {
        recordings.push(row?);
    }
    Ok(recordings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn new_recording(file_name: &str, project_id: Option<String>) -> NewRecording {
        NewRecording {
            file_name: file_name.to_string(),
            source_path: format!("/tmp/{file_name}"),
            project_id,
            audio_duration_secs: None,
        }
    }

    #[test]
    fn creates_and_lists_projects() {
        let conn = setup();
        let project = create_project(&conn, "  Board Meetings  ", Some("weekly")).unwrap();

        assert_eq!(project.name, "Board Meetings");
        assert_eq!(project.description.as_deref(), Some("weekly"));
        assert_eq!(project.recording_count, 0);
        assert_eq!(list_projects(&conn).unwrap().len(), 1);
    }

    #[test]
    fn rejects_empty_project_name() {
        let conn = setup();
        assert!(create_project(&conn, "   ", None).is_err());
    }

    #[test]
    fn adds_and_lists_recordings() {
        let conn = setup();
        let project = create_project(&conn, "P", None).unwrap();
        let recording = add_recording(&conn, &new_recording("a.mp3", Some(project.id.clone()))).unwrap();

        assert_eq!(recording.status, "pending");
        assert_eq!(recording.file_name, "a.mp3");

        let scoped = list_recordings(&conn, Some(&project.id)).unwrap();
        assert_eq!(scoped.len(), 1);
        assert_eq!(list_recordings(&conn, None).unwrap().len(), 1);
        assert_eq!(list_projects(&conn).unwrap()[0].recording_count, 1);
    }

    #[test]
    fn updates_status_metadata_and_saves_text() {
        let conn = setup();
        let recording = add_recording(&conn, &new_recording("a.wav", None)).unwrap();

        update_recording_status(&conn, &recording.id, "transcribing").unwrap();
        update_recording_metadata(&conn, &recording.id, Some(12.5), Some("en")).unwrap();
        save_transcript(&conn, &recording.id, "hello world").unwrap();
        save_summary(&conn, &recording.id, "# Summary").unwrap();

        let fetched = get_recording(&conn, &recording.id).unwrap();
        assert_eq!(fetched.status, "transcribing");
        assert_eq!(fetched.audio_duration_secs, Some(12.5));
        assert_eq!(fetched.detected_language.as_deref(), Some("en"));
        assert_eq!(fetched.transcript_raw.as_deref(), Some("hello world"));
        assert_eq!(fetched.summary_markdown.as_deref(), Some("# Summary"));
    }

    #[test]
    fn metadata_update_keeps_existing_values_on_none() {
        let conn = setup();
        let recording = add_recording(&conn, &new_recording("a.wav", None)).unwrap();
        update_recording_metadata(&conn, &recording.id, Some(5.0), Some("de")).unwrap();
        update_recording_metadata(&conn, &recording.id, None, None).unwrap();

        let fetched = get_recording(&conn, &recording.id).unwrap();
        assert_eq!(fetched.audio_duration_secs, Some(5.0));
        assert_eq!(fetched.detected_language.as_deref(), Some("de"));
    }

    #[test]
    fn assigns_recording_to_project() {
        let conn = setup();
        let project = create_project(&conn, "P", None).unwrap();
        let recording = add_recording(&conn, &new_recording("a.mp3", None)).unwrap();

        assign_recording_to_project(&conn, &recording.id, Some(&project.id)).unwrap();
        assert_eq!(get_recording(&conn, &recording.id).unwrap().project_id, Some(project.id.clone()));
        assert_eq!(list_projects(&conn).unwrap()[0].recording_count, 1);

        assign_recording_to_project(&conn, &recording.id, None).unwrap();
        assert_eq!(get_recording(&conn, &recording.id).unwrap().project_id, None);
    }

    #[test]
    fn searches_by_keyword_project_and_tag() {
        let conn = setup();
        let alpha = create_project(&conn, "Alpha", None).unwrap();
        let beta = create_project(&conn, "Beta", None).unwrap();

        let meeting = add_recording(&conn, &new_recording("meeting.mp3", Some(alpha.id.clone()))).unwrap();
        let notes = add_recording(&conn, &new_recording("notes.mp3", Some(beta.id.clone()))).unwrap();

        save_transcript(&conn, &meeting.id, "we discussed the budget").unwrap();
        save_summary(&conn, &meeting.id, "## budget summary").unwrap();
        save_transcript(&conn, &notes.id, "engineering standup").unwrap();
        add_tag(&conn, &meeting.id, "sales").unwrap();

        assert_eq!(search_recordings(&conn, None, None, None).unwrap().len(), 2);
        assert_eq!(search_recordings(&conn, Some("budget"), None, None).unwrap().len(), 1);
        assert_eq!(search_recordings(&conn, None, Some(&alpha.id), None).unwrap().len(), 1);
        assert_eq!(search_recordings(&conn, None, None, Some("sales")).unwrap().len(), 1);
        assert_eq!(
            search_recordings(&conn, Some("budget"), None, Some("sales")).unwrap().len(),
            1
        );
        assert_eq!(search_recordings(&conn, Some("missing"), None, None).unwrap().len(), 0);
    }

    #[test]
    fn lists_and_deletes_tags() {
        let conn = setup();
        let recording = add_recording(&conn, &new_recording("a.mp3", None)).unwrap();
        add_tag(&conn, &recording.id, "urgent").unwrap();
        add_tag(&conn, &recording.id, "followup").unwrap();

        let tags = list_tags(&conn, &recording.id).unwrap();
        assert_eq!(tags.len(), 2);

        delete_recording(&conn, &recording.id).unwrap();
        assert!(list_tags(&conn, &recording.id).unwrap().is_empty());
        assert!(get_recording(&conn, &recording.id).is_err());
    }

    #[test]
    fn renames_project_and_recording() {
        let conn = setup();
        let project = create_project(&conn, "Old", None).unwrap();
        assert_eq!(rename_project(&conn, &project.id, "  New  ").unwrap().name, "New");
        assert!(rename_project(&conn, &project.id, "   ").is_err());

        let recording = add_recording(&conn, &new_recording("a.mp3", None)).unwrap();
        assert_eq!(rename_recording(&conn, &recording.id, "b.mp3").unwrap().file_name, "b.mp3");
        assert!(rename_recording(&conn, &recording.id, "").is_err());
    }

    #[test]
    fn deletes_a_single_tag() {
        let conn = setup();
        let recording = add_recording(&conn, &new_recording("a.mp3", None)).unwrap();
        add_tag(&conn, &recording.id, "keep").unwrap();
        add_tag(&conn, &recording.id, "drop").unwrap();

        let tags = list_tags(&conn, &recording.id).unwrap();
        let tag = tags.iter().find(|tag| tag.tag_name == "drop").unwrap();
        delete_tag(&conn, tag.id).unwrap();

        let remaining = list_tags(&conn, &recording.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].tag_name, "keep");
    }

    #[test]
    fn deleting_project_cascades_recordings_and_tags() {
        let conn = setup();
        let project = create_project(&conn, "P", None).unwrap();
        let recording = add_recording(&conn, &new_recording("a.mp3", Some(project.id.clone()))).unwrap();
        add_tag(&conn, &recording.id, "x").unwrap();

        delete_project(&conn, &project.id).unwrap();

        assert!(list_projects(&conn).unwrap().is_empty());
        assert!(list_recordings(&conn, None).unwrap().is_empty());
        assert!(list_tags(&conn, &recording.id).unwrap().is_empty());
    }
}
