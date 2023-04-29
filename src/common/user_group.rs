use crate::prelude::*;

const MAX_NAME_LEN: usize = 32;

#[derive(PartialEq, Eq)]
#[cfg_attr(test, derive(Clone, Copy))]
pub struct IdName {
    len: u8,
    data: [u8; MAX_NAME_LEN],
}

impl std::fmt::Debug for IdName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdName")
            .field("len", &self.len)
            .field("data", &BinaryToDebug(self))
            .finish()
    }
}

impl IdName {
    #[cfg(test)]
    pub fn new(name_data: &[u8]) -> Self {
        let mut data = [0_u8; MAX_NAME_LEN];
        copy_to_start(&mut data, name_data);
        Self {
            len: truncate_usize_u8(name_data.len()),
            data,
        }
    }
}

impl std::ops::Deref for IdName {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data[..zero_extend_u8_usize(self.len)]
    }
}

// There's relatively few of these (like on the scale of ones to tens), and it's only looked
// through once every minute. It doesn't need to be an entire hash map.
#[derive(Debug, PartialEq)]
pub struct IdTable {
    entries: Box<[(u32, IdName)]>,
}

impl IdTable {
    #[cfg(test)]
    pub fn from_entries(slice: &[(u32, IdName)]) -> Self {
        Self {
            entries: slice.into(),
        }
    }

    pub fn lookup_name(&self, search_key: &[u8]) -> Option<u32> {
        self.entries
            .iter()
            .find_map(|(id, name)| (&**name == search_key).then_some(*id))
    }

    pub fn lookup_id(&self, search_id: u32) -> Option<&IdName> {
        self.entries
            .iter()
            .find_map(|(id, name)| (*id == search_id).then_some(name))
    }
}

#[derive(Debug, PartialEq)]
pub struct UidGidTable {
    pub uids: IdTable,
    pub gids: IdTable,
}

impl UidGidTable {
    pub const fn new(uid_table: IdTable, gid_table: IdTable) -> UidGidTable {
        UidGidTable {
            uids: uid_table,
            gids: gid_table,
        }
    }

    /// Pass the result of `parse_etc_passwd_etc_group` as `user_group_count` and the decoded
    /// result as `user_group_buf`. Don't use this for general search.
    ///
    /// This may seem out of place functionally, but this is where the data is defined, and I want
    /// to be able to better maintain the inner data structure.
    #[cold]
    pub fn lookup_user_group(&self, search_user: &[u8], search_group: &[u8]) -> Option<UserGroup> {
        match (
            self.uids.lookup_name(search_user),
            self.gids.lookup_name(search_group),
        ) {
            (Some(uid), Some(gid)) => Some(UserGroup { uid, gid }),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UserGroup {
    pub uid: u32,
    pub gid: u32,
}

/*
This is a very simple `/etc/passwd` and `/etc/group` parser that just parses out what I need.
Note: this also leverages the fact the first 3 groups are identical:
- User/group name
- Password
- User/group ID

The full format for each line is actually this:

- /etc/passwd: `user:pass:uid:gid:comment:home:login`
  - `user`: The username.
  - `pass`: The password. It's usually `x` on most systems, with the password itself stored in
    hashed form in `/etc/shadow`.
  - `uid`: The numeric user ID accepted by most syscalls to refer to this user.
  - `gid`: The primary group ID corresponding to this user.
  - `comment`: A field for arbitrary comments about the user. Rarely useful in practice.
  - `home`: The home directory. Usually `/home/user` for interactive users, and usually empty for
    machine users (in which it defaults to `/`).
  - `login`: The login shell. Usually the default system shell for interactive users, and should
    generally be set to `/sbin/nologin` for machine users.

- /etc/group: `group:pass:gid:users...`
  - `group`: The group.
  - `pass`: The password. If present, it's usually `x` on most systems, with the password itself
    stored in hashed form in `/etc/shadow`, but it's almost never used.
  - `gid`: The numeric group ID accepted by most syscalls to refer to this group.
  - `users...`: Zero or more users separated by commas, representing its members.
*/

enum ParserState {
    Drop,
    Start,
    Name(IdName),
    NameEnd(IdName),
    Password(IdName),
    IdStart(IdName, u32),
    IdPart(IdName, u32),
}

pub struct PasswdGroupParser {
    state: ParserState,
    names: Vec<(u32, IdName)>,
}

impl PasswdGroupParser {
    pub const fn new() -> Self {
        Self {
            state: ParserState::Start,
            names: Vec::new(),
        }
    }

    pub fn extract(self) -> IdTable {
        IdTable {
            entries: self.names.into(),
        }
    }

    fn insert_name(&mut self, name: IdName, id: u32) -> bool {
        for item in self.names.iter_mut() {
            if item.0 == id {
                item.1 = name;
                return true;
            }
        }

        if self.names.try_reserve(1).is_err() {
            return false;
        }

        self.names.push((id, name));
        true
    }

    pub fn consume(&mut self, chunk: &[u8]) -> bool {
        for &ch in chunk {
            match (replace(&mut self.state, ParserState::Drop), ch) {
                (_, b'\n') => self.state = ParserState::Start,

                (ParserState::Start, b'_' | b'A'..=b'Z' | b'a'..=b'z') => {
                    let mut data = [0; MAX_NAME_LEN];
                    data[0] = ch;
                    self.state = ParserState::Name(IdName { len: 1, data });
                }

                (ParserState::Name(mut name), b'$')
                    if name.len < truncate_usize_u8(MAX_NAME_LEN) =>
                {
                    name.data[zero_extend_u8_usize(name.len)] = ch;
                    name.len = name.len.wrapping_add(1);
                    self.state = ParserState::NameEnd(name);
                }

                (
                    ParserState::Name(mut name),
                    b'0'..=b'9' | b'_' | b'-' | b'A'..=b'Z' | b'a'..=b'z',
                ) if name.len < truncate_usize_u8(MAX_NAME_LEN) => {
                    name.data[zero_extend_u8_usize(name.len)] = ch;
                    name.len = name.len.wrapping_add(1);
                    self.state = ParserState::Name(name);
                }

                (ParserState::Name(name) | ParserState::NameEnd(name), b':') => {
                    self.state = ParserState::Password(name);
                }

                (ParserState::Password(name), b':') => {
                    self.state = ParserState::IdStart(name, 0);
                }

                (ParserState::Password(name), _) => {
                    self.state = ParserState::Password(name);
                }

                (ParserState::IdPart(name, acc), b':') => {
                    if !self.insert_name(name, acc) {
                        return false;
                    }
                    self.state = ParserState::Drop;
                }

                (ParserState::IdStart(name, acc) | ParserState::IdPart(name, acc), _) => {
                    match parse_u32_digit(acc, ch) {
                        Some(acc) => self.state = ParserState::IdPart(name, acc),
                        None => self.state = ParserState::Drop,
                    }
                }

                (_, _) => self.state = ParserState::Drop,
            }
        }

        true
    }
}
