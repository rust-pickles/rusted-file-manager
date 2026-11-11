/// Case-insensitive subsequence score. Lower is better; contiguous and prefix matches win.
pub fn fuzzy_score(query:&str,candidate:&str)->Option<i32>{
    if query.is_empty(){return Some(0);}let q:Vec<char>=query.chars().flat_map(char::to_lowercase).collect();let c:Vec<char>=candidate.chars().flat_map(char::to_lowercase).collect();
    let mut qi=0;let mut previous=None;let mut gaps=0;let mut first=0;
    for(i,ch)in c.iter().enumerate(){if qi<q.len()&&*ch==q[qi]{if qi==0{first=i;}if let Some(p)=previous{gaps+=i-p-1;}previous=Some(i);qi+=1;}}
    // Rank by gap count, start position and how much of the name was consumed.
    // Penalising total name length instead made "file" score worse in "file.rs"
    // than in "f-i-l-e.rs", i.e. an exact prefix lost to a scattered match.
    let span=previous.map(|last|last+1-first).unwrap_or(0);
    (qi==q.len()).then_some((gaps*4+first*2+span.saturating_sub(q.len()))as i32)
}
pub fn filter_sorted<T>(query:&str,items:impl IntoIterator<Item=T>,name:impl Fn(&T)->&str,limit:usize)->Vec<T>{let mut matches:Vec<_>=items.into_iter().filter_map(|item|fuzzy_score(query,name(&item)).map(|score|(score,item))).collect();matches.sort_by(|a,b|a.0.cmp(&b.0));matches.truncate(limit);matches.into_iter().map(|(_,item)|item).collect()}
#[cfg(test)]mod tests{use super::*;#[test]fn subsequence_and_order(){// Initials of "Rusted File Manager". "rfc" was wrong: there is no "c" in
    // the candidate at all, and a subsequence match must require every character.
    assert!(fuzzy_score("rfm","RustedFileManager").is_some());assert!(fuzzy_score("xyz","Rusted").is_none());assert!(fuzzy_score("file","file.rs").unwrap()<fuzzy_score("file","f-i-l-e.rs").unwrap());}}
