#[cfg(target_family = "unix")]
pub fn user_id() -> u32 {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self")
        .map(|m| m.uid())
        .expect("/proc/self should exist")
}

pub fn get_passwd(uid: &[u32]) -> String {
    filter_sys_db("/etc/passwd", 2, uid)
}

pub fn get_groups(gid: &[u32]) -> String {
    filter_sys_db("/etc/group", 2, gid)
}

pub fn filter_sys_db(path: &str, ind: usize, ids: &[u32]) -> String {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| {
            line.split(':')
                .nth(ind)
                .and_then(|s| s.parse::<u32>().ok())
                .is_some_and(|id| ids.contains(&id))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
