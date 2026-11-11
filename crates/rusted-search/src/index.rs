use crate::{fuzzy_score,SearchHit,SearchQuery};
use rusqlite::{params,Connection,OptionalExtension};
use rusted_fs::entry_from_metadata;
use std::{fs,path::{Path,PathBuf},sync::atomic::{AtomicBool,Ordering},time::Instant};
use thiserror::Error;
#[derive(Debug,Error)]pub enum SearchError{#[error("search database: {0}")]Database(#[from]rusqlite::Error),#[error("index I/O for {path}: {source}")]Io{path:PathBuf,#[source]source:std::io::Error}}
#[derive(Debug,Clone,Default,PartialEq,Eq)]pub struct IndexStats{pub visited:usize,pub indexed:usize,pub skipped:usize,pub elapsed_ms:u128,pub cancelled:bool}
pub struct SearchIndex{connection:Connection}
impl SearchIndex{
    pub fn open(path:&Path)->Result<Self,SearchError>{let connection=Connection::open(path)?;Self::initialize(connection)}
    pub fn in_memory()->Result<Self,SearchError>{Self::initialize(Connection::open_in_memory()?)}
    fn initialize(connection:Connection)->Result<Self,SearchError>{connection.execute_batch("PRAGMA journal_mode=WAL;PRAGMA synchronous=NORMAL;PRAGMA foreign_keys=ON;CREATE TABLE IF NOT EXISTS entries(path TEXT PRIMARY KEY,name TEXT NOT NULL,size INTEGER NOT NULL,modified INTEGER,kind TEXT NOT NULL,extension TEXT,root TEXT NOT NULL);CREATE INDEX IF NOT EXISTS entries_name ON entries(name);CREATE INDEX IF NOT EXISTS entries_root ON entries(root);CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(name,path,content='entries',content_rowid='rowid');CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN INSERT INTO entries_fts(rowid,name,path) VALUES(new.rowid,new.name,new.path);END;CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN INSERT INTO entries_fts(entries_fts,rowid,name,path) VALUES('delete',old.rowid,old.name,old.path);END;CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN INSERT INTO entries_fts(entries_fts,rowid,name,path) VALUES('delete',old.rowid,old.name,old.path);INSERT INTO entries_fts(rowid,name,path) VALUES(new.rowid,new.name,new.path);END;CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY,indexed_at INTEGER NOT NULL);")?;Ok(Self{connection})}
    pub fn index_root(&self,root:&Path,exclusions:&[PathBuf],cancel:&AtomicBool,mut on_progress:impl FnMut(IndexStats))->Result<IndexStats,SearchError>{
        let started=Instant::now();let mut stats=IndexStats::default();let transaction=self.connection.unchecked_transaction()?;
        walk(root,root,exclusions,cancel,&transaction,&mut stats)?;let now=chrono::Utc::now().timestamp();transaction.execute("INSERT INTO roots(path,indexed_at) VALUES(?1,?2) ON CONFLICT(path) DO UPDATE SET indexed_at=excluded.indexed_at",params![root.to_string_lossy(),now])?;transaction.commit()?;stats.elapsed_ms=started.elapsed().as_millis();stats.cancelled=cancel.load(Ordering::Relaxed);on_progress(stats.clone());Ok(stats)
    }
    pub fn upsert_path(&self,path:&Path,root:&Path)->Result<(),SearchError>{let metadata=fs::symlink_metadata(path).map_err(|source|SearchError::Io{path:path.into(),source})?;let file_type=metadata.file_type();let entry=entry_from_metadata(path,&metadata,file_type);insert(&self.connection,&entry,root)?;Ok(())}
    pub fn remove_path(&self,path:&Path)->Result<(),SearchError>{self.connection.execute("DELETE FROM entries WHERE path=?1 OR path LIKE ?2",params![path.to_string_lossy(),format!("{}%",escape_like(&path.to_string_lossy()))])?;Ok(())}
    pub fn search(&self,query:&SearchQuery)->Result<Vec<SearchHit>,SearchError>{
        let limit=if query.limit==0{100}else{query.limit.min(1000)};let text=query.text.trim();let mut hits=Vec::new();
        if !text.is_empty(){
            let fts=text.split_whitespace().map(|token|format!("\"{}\"*",token.replace('"',"\"\""))).collect::<Vec<_>>().join(" AND ");
            let mut statement=self.connection.prepare("SELECT e.path,e.name,e.size,e.modified,e.kind FROM entries_fts f JOIN entries e ON e.rowid=f.rowid WHERE entries_fts MATCH ?1 LIMIT ?2")?;
            let rows=statement.query_map(params![fts,limit as i64],row_to_hit)?;for row in rows{let mut hit=row?;hit.score=fuzzy_score(text,&hit.name).unwrap_or(1000);hits.push(hit);}
        }
        if hits.len()<limit{
            let mut statement=self.connection.prepare("SELECT path,name,size,modified,kind FROM entries ORDER BY name")?;
            let rows=statement.query_map([],row_to_hit)?;for row in rows{let hit=row?;if !matches(&hit,query){continue;}if !text.is_empty(){let Some(score)=fuzzy_score(text,&hit.name)else{continue};if hits.iter().any(|h|h.path==hit.path){continue;}let mut hit=hit;hit.score=score;hits.push(hit);}else if !hits.iter().any(|h|h.path==hit.path){hits.push(hit);}if hits.len()>=limit{break;}}
        }
        hits.retain(|h|matches(h,query));hits.sort_by(|a,b|a.score.cmp(&b.score).then_with(||a.name.cmp(&b.name)));hits.truncate(limit);Ok(hits)
    }
    pub fn count(&self)->Result<u64,SearchError>{Ok(self.connection.query_row("SELECT COUNT(*) FROM entries",[],|r|r.get(0))?)}
    pub fn root_indexed_at(&self,root:&Path)->Result<Option<i64>,SearchError>{Ok(self.connection.query_row("SELECT indexed_at FROM roots WHERE path=?1",[root.to_string_lossy()],|r|r.get(0)).optional()?)}
}
fn row_to_hit(row:&rusqlite::Row<'_>)->rusqlite::Result<SearchHit>{Ok(SearchHit{path:PathBuf::from(row.get::<_,String>(0)?),name:row.get(1)?,size:row.get::<_,i64>(2)?.max(0)as u64,modified:row.get(3)?,kind:row.get(4)?,score:0})}
fn matches(hit:&SearchHit,q:&SearchQuery)->bool{
    if !q.roots.is_empty()&&!q.roots.iter().any(|r|hit.path.starts_with(r)){return false;}
    if q.path_contains.as_ref().is_some_and(|p|!hit.path.to_string_lossy().to_lowercase().contains(&p.to_lowercase())){return false;}
    if q.extension.as_ref().is_some_and(|e|hit.path.extension().map(|x|x.to_string_lossy().eq_ignore_ascii_case(e.trim_start_matches('.')))!=Some(true)){return false;}
    if q.kind.as_ref().is_some_and(|k|!hit.kind.eq_ignore_ascii_case(k)){return false;}
    if q.min_size.is_some_and(|s|hit.size<s)||q.max_size.is_some_and(|s|hit.size>s){return false;}
    if q.modified_after.is_some_and(|d|hit.modified.is_none_or(|m|m<d.timestamp()))||q.modified_before.is_some_and(|d|hit.modified.is_none_or(|m|m>d.timestamp())){return false;}true
}
fn insert(connection:&Connection,entry:&rusted_core::FileEntry,root:&Path)->Result<(),rusqlite::Error>{connection.execute("INSERT INTO entries(path,name,size,modified,kind,extension,root) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(path) DO UPDATE SET name=excluded.name,size=excluded.size,modified=excluded.modified,kind=excluded.kind,extension=excluded.extension,root=excluded.root",params![entry.path.to_string_lossy(),entry.name,entry.size as i64,entry.modified.map(|d|d.timestamp()),format!("{:?}",entry.kind).to_lowercase(),entry.extension,root.to_string_lossy()])?;Ok(())}
fn walk(root:&Path,path:&Path,exclusions:&[PathBuf],cancel:&AtomicBool,connection:&Connection,stats:&mut IndexStats)->Result<(),SearchError>{
    if cancel.load(Ordering::Relaxed)||exclusions.iter().any(|e|path.starts_with(e)){stats.skipped+=1;return Ok(());}stats.visited+=1;let metadata=fs::symlink_metadata(path).map_err(|source|SearchError::Io{path:path.into(),source})?;let file_type=metadata.file_type();let entry=entry_from_metadata(path,&metadata,file_type);insert(connection,&entry,root)?;stats.indexed+=1;
    if metadata.is_dir()&&!file_type.is_symlink()&&rusted_core::EntryKind::Junction!=entry.kind{let children=fs::read_dir(path).map_err(|source|SearchError::Io{path:path.into(),source})?;for child in children{let child=child.map_err(|source|SearchError::Io{path:path.into(),source})?;walk(root,&child.path(),exclusions,cancel,connection,stats)?;if cancel.load(Ordering::Relaxed){break;}}}Ok(())
}
fn escape_like(value:&str)->String{value.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_")}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("rusted-search-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn indexing_records_the_root_and_finds_files() {
        let root = temp_root();
        fs::write(root.join("quarterly-report.txt"), b"x").unwrap();
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("nested").join("invoice.txt"), b"y").unwrap();

        let index = SearchIndex::in_memory().unwrap();
        let stats = index.index_root(&root, &[], &AtomicBool::new(false), |_| {}).unwrap();
        assert!(stats.indexed >= 3, "expected root + children, got {stats:?}");
        assert!(!stats.cancelled);
        assert!(index.root_indexed_at(&root).unwrap().is_some(), "root timestamp must be stored");

        let hits = index
            .search(&SearchQuery { text: "quarterly".into(), limit: 10, ..Default::default() })
            .unwrap();
        assert!(hits.iter().any(|hit| hit.name == "quarterly-report.txt"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn exclusions_and_cancellation_are_honoured() {
        let root = temp_root();
        fs::create_dir_all(root.join("skip-me")).unwrap();
        fs::write(root.join("skip-me").join("hidden.txt"), b"x").unwrap();
        fs::write(root.join("kept.txt"), b"x").unwrap();

        let index = SearchIndex::in_memory().unwrap();
        let excluded = vec![root.join("skip-me")];
        index.index_root(&root, &excluded, &AtomicBool::new(false), |_| {}).unwrap();
        let hits = index.search(&SearchQuery { text: "hidden".into(), limit: 10, ..Default::default() }).unwrap();
        assert!(hits.is_empty(), "excluded subtree must not be indexed");

        let cancel = AtomicBool::new(true);
        let stats = index.index_root(&root, &[], &cancel, |_| {}).unwrap();
        assert!(stats.cancelled || stats.indexed <= 1);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn removal_and_filters_narrow_results() {
        let root = temp_root();
        fs::write(root.join("note.md"), vec![0u8; 10]).unwrap();
        fs::write(root.join("note.txt"), vec![0u8; 5000]).unwrap();
        let index = SearchIndex::in_memory().unwrap();
        index.index_root(&root, &[], &AtomicBool::new(false), |_| {}).unwrap();

        let by_extension = index
            .search(&SearchQuery { text: "note".into(), extension: Some("md".into()), limit: 10, ..Default::default() })
            .unwrap();
        assert_eq!(by_extension.len(), 1);
        assert_eq!(by_extension[0].name, "note.md");

        let by_size = index
            .search(&SearchQuery { text: "note".into(), min_size: Some(1000), limit: 10, ..Default::default() })
            .unwrap();
        assert_eq!(by_size.len(), 1);
        assert_eq!(by_size[0].name, "note.txt");

        index.remove_path(&root.join("note.md")).unwrap();
        let after = index.search(&SearchQuery { text: "note".into(), limit: 10, ..Default::default() }).unwrap();
        assert!(!after.iter().any(|hit| hit.name == "note.md"));
        fs::remove_dir_all(&root).unwrap();
    }
}
