use crate::prelude::*;

const MAX_NAME_LEN: usize = 32;

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArbitraryNamePart(u8);

// `Vec::from_iter(1..=MAX_NAME_LEN)`, but static.
#[cfg(test)]
static NAME_LENGTHS: &[usize] = &{
    let mut key_lengths = [1; MAX_NAME_LEN];
    let mut i = 0;
    while i < key_lengths.len() {
        key_lengths[i] = i + 1;
        i += 1;
    }
    key_lengths
};

// Note: these must remain sorted by code point.
#[cfg(test)]
static NAME_START_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
#[cfg(test)]
static NAME_PART_CHARS: &[u8] = b"-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

#[cfg(test)]
fn shrink_array(array: &[u8], prev: u8) -> impl Iterator<Item = u8> + '_ {
    array.iter().cloned().take_while(move |c| *c < prev)
}

#[cfg(test)]
impl Arbitrary for ArbitraryNamePart {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(*g.choose(NAME_PART_CHARS).unwrap())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(shrink_array(NAME_PART_CHARS, self.0).map(Self))
    }
}

#[cfg(test)]
fn is_valid_name(bytes: &[u8]) -> bool {
    if bytes.len() > MAX_NAME_LEN {
        return false;
    }

    let Some((head, tail)) = bytes.split_first() else {
            return false;
        };

    matches!(*head, b'_' | b'A'..=b'Z' | b'a'..=b'z')
        && tail
            .iter()
            .all(|b| matches!(*b, b'0'..=b'9' | b'_' | b'-' | b'A'..=b'Z' | b'a'..=b'z'))
}

#[cfg(test)]
impl Arbitrary for IdName {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut result = Vec::new();
        let len = *g.choose(NAME_LENGTHS).unwrap();

        result.push(*g.choose(NAME_START_CHARS).unwrap());

        for _ in 2..len {
            result.push(*g.choose(NAME_PART_CHARS).unwrap());
        }

        Self::new(&result)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let data = self.clone();
        let (&head, tail) = data.split_first().unwrap();
        let tail = Vec::from_iter(tail.iter().cloned().map(ArbitraryNamePart));

        Box::new(shrink_array(NAME_START_CHARS, head).flat_map(move |head| {
            tail.shrink().map(move |v| {
                Self::new(&Vec::from_iter(
                    std::iter::once(head).chain(v.into_iter().map(|c| c.0)),
                ))
            })
        }))
    }
}

// There's relatively few of these (like on the scale of ones to tens), and it's only looked
// through once every minute. It doesn't need to be an entire hash map.
#[derive(Debug, Clone, PartialEq)]
pub struct IdTable {
    entries: Box<[(u32, IdName)]>,
}

impl IdTable {
    #[cfg(test)]
    pub fn from_entries(slice: &[(u32, IdName)]) -> Self {
        Self {
            entries: slice.to_vec().into(),
        }
    }

    pub fn lookup_key(&self, search_key: &[u8]) -> Option<u32> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct UidGidTable {
    uid_table: IdTable,
    gid_table: IdTable,
}

impl UidGidTable {
    /// This is `unsafe` because it's not validated that `user_group_data` is in fact a valid
    /// serialized sequence of a user table + group table.
    pub const fn new(uid_table: IdTable, gid_table: IdTable) -> UidGidTable {
        UidGidTable {
            uid_table,
            gid_table,
        }
    }

    pub fn lookup_uid(&self, uid: Uid) -> Option<&IdName> {
        self.uid_table.lookup_id(uid.into())
    }

    pub fn lookup_gid(&self, gid: Gid) -> Option<&IdName> {
        self.gid_table.lookup_id(gid.into())
    }

    /// Pass the result of `parse_etc_passwd_etc_group` as `user_group_count` and the decoded
    /// result as `user_group_buf`. Don't use this for general search.
    ///
    /// This may seem out of place functionally, but this is where the data is defined, and I want
    /// to be able to better maintain the inner data structure.
    #[cold]
    pub fn lookup_user_group(
        &self,
        search_user: &[u8],
        search_group: &[u8],
    ) -> (Option<Uid>, Option<Gid>) {
        (
            self.uid_table.lookup_key(search_user).map(Uid::from),
            self.gid_table.lookup_key(search_group).map(Gid::from),
        )
    }
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

fn insert_name(names: &mut Vec<(u32, IdName)>, id: u32, name: &[u8]) {
    let name = IdName::new(name);

    for item in names.iter_mut() {
        if item.0 == id {
            item.1 = name;
            return;
        }
    }

    names.push((id, name));
}

pub fn parse_etc_passwd_etc_group(data: &[u8]) -> IdTable {
    let mut iter = data.iter().copied().enumerate();
    let mut names = Vec::new();

    'done: loop {
        'parse_line: loop {
            match iter.next() {
                None => break 'done,
                Some((name_start, b'_' | b'A'..=b'Z' | b'a'..=b'z')) => {
                    let mut name_len = Wrapping(1);

                    'parse_name: loop {
                        match iter.next() {
                            None => break 'done,
                            Some((_, b':')) => break 'parse_name,
                            Some(_) if name_len >= Wrapping(MAX_NAME_LEN) => break 'parse_line,
                            Some((_, b'$')) => {
                                name_len += 1;
                                match iter.next() {
                                    None => break 'done,
                                    Some((_, b':')) => break 'parse_name,
                                    Some(_) => break 'parse_line,
                                }
                            }
                            Some((_, b'0'..=b'9' | b'_' | b'-' | b'A'..=b'Z' | b'a'..=b'z')) => {
                                name_len += 1;
                            }
                            Some((_, b'\n')) => continue 'parse_line,
                            Some(_) => break 'parse_line,
                        }
                    }

                    'skip_password: loop {
                        match iter.next() {
                            None => break 'done,
                            Some((_, b':')) => break 'skip_password,
                            Some((_, b'\n')) => continue 'parse_line,
                            Some(_) => continue 'skip_password,
                        }
                    }

                    let mut id = match iter.next() {
                        None => break 'done,
                        Some((_, byte @ b'0'..=b'9')) => match parse_u32_digit(0, byte) {
                            Some(id) => id,
                            None => break 'parse_line,
                        },
                        Some((_, b'\n')) => continue 'parse_line,
                        Some(_) => break 'parse_line,
                    };

                    loop {
                        match iter.next() {
                            None => break 'done,
                            Some((_, byte @ b'0'..=b'9')) => match parse_u32_digit(id, byte) {
                                Some(result) => id = result,
                                None => break 'parse_line,
                            },
                            Some((_, b':')) => {
                                let name_end = (Wrapping(name_start) + name_len).0;
                                insert_name(&mut names, id, &data[name_start..name_end]);
                                break 'parse_line;
                            }
                            Some((_, b'\n')) => continue 'parse_line,
                            Some(_) => break 'parse_line,
                        }
                    }
                }
                Some((_, b'\n')) => continue 'parse_line,
                Some(_) => break 'parse_line,
            }
        }

        'drop: loop {
            match iter.next() {
                None => break 'done,
                Some((_, b'\n')) => break 'drop,
                Some(_) => {}
            }
        }
    }

    IdTable {
        entries: names.into(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // These tests are ignored by Miri as 1. they take a while and 2. the things they're checking
    // are only used in test anyways.

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_name_start_chars_are_sorted() {
        let mut sorted = NAME_START_CHARS.to_vec();
        sorted.sort();
        assert_eq!(&*sorted, NAME_START_CHARS);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_name_part_chars_are_sorted() {
        let mut sorted = NAME_PART_CHARS.to_vec();
        sorted.sort();
        assert_eq!(&*sorted, NAME_PART_CHARS);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_name_chars_are_correct() {
        for first in 0..=u8::MAX {
            for second in 0..=u8::MAX {
                let expected_is_valid =
                    NAME_START_CHARS.contains(&first) && NAME_PART_CHARS.contains(&second);

                assert!(
                    is_valid_name(&[first, second]) == expected_is_valid,
                    "\"{}\"",
                    BinaryToDisplay(&[first, second])
                );
            }
        }
    }
}
