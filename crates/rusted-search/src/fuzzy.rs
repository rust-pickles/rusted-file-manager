/// Case-insensitive subsequence score. Lower is better; contiguous and prefix matches win.
pub fn fuzzy_score(query:&str,candidate:&str)->Option<i32>{
    if query.is_empty(){return Some(0);}let q:Vec<char>=query.chars().flat_map(char::to_lowercase).collect();let c:Vec<char>=candidate.chars().flat_map(char::to_lowercase).collect();
    let mut qi=0;let mut previous=None;let mut gaps=0;let mut first=0;
    for(i,ch)in c.iter().enumerate(){if qi<q.len()&&*ch==q[qi]{if qi==0{first=i;}if let Some(p)=previous{gaps+=i-p-1;}previous=Some(i);qi+=1;}}
    (qi==q.len()).then_some((gaps*4+first*2+c.len().saturating_sub(q.len())/8)as i32)
}
pub fn filter_sorted<T>(query:&str,items:impl IntoIterator<Item=T>,name:impl Fn(&T)->&str,limit:usize)->Vec<T>{let mut matches:Vec<_>=items.into_iter().filter_map(|item|fuzzy_score(query,name(&item)).map(|score|(score,item))).collect();matches.sort_by(|a,b|a.0.cmp(&b.0));matches.truncate(limit);matches.into_iter().map(|(_,item)|item).collect()}
#[cfg(test)]mod tests{use super::*;#[test]fn subsequence_and_order(){assert!(fuzzy_score("rfc","RustedFileManager").is_some());assert!(fuzzy_score("xyz","Rusted").is_none());assert!(fuzzy_score("file","file.rs").unwrap()<fuzzy_score("file","f-i-l-e.rs").unwrap());}}
