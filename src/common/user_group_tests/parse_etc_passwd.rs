use crate::common::parse_etc_passwd_etc_group as parse;
use crate::common::IdTable;

fn name(value: &[u8]) -> crate::prelude::IdName {
    crate::prelude::IdName::new(value)
}

#[test]
fn detects_empty_names() {
    assert_eq!(parse(b""), IdTable::from_entries(&[]));
}

#[test]
fn allows_single_character_names() {
    assert_eq!(
        parse(b"a:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"a"))])
    );
}

#[test]
fn allows_empty_passwords() {
    assert_eq!(
        parse(b"a::123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"a"))])
    );
}

#[test]
fn disallows_usernames_not_followed_by_a_colon() {
    assert_eq!(parse(b"a"), IdTable::from_entries(&[]));
}

#[test]
fn disallows_empty_usernames() {
    assert_eq!(
        parse(b":x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_break_after_password() {
    assert_eq!(parse(b"a:x\n"), IdTable::from_entries(&[]));
}

#[test]
fn disallows_empty_ids() {
    assert_eq!(
        parse(b"a:x::123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_break_before_id() {
    assert_eq!(parse(b"a:x:\n"), IdTable::from_entries(&[]));
}

#[test]
fn allows_break_after_id() {
    assert_eq!(
        parse(b"a:x:1:\n"),
        IdTable::from_entries(&[(1, name(b"a"))])
    );
}

#[test]
fn allows_small_alphanumeric_names() {
    assert_eq!(
        parse(b"abc123:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abc123"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_ending_with_a_dollar_sign() {
    assert_eq!(
        parse(b"abc123$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abc123$"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names() {
    assert_eq!(
        parse(b"abcdefghijklmnopqrstuvwxyz123456:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz123456"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_ending_with_a_dollar_sign() {
    assert_eq!(
        parse(b"abcdefghijklmnopqrstuvwxyz12345$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz12345$"))]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names() {
    assert_eq!(
        parse(b"abcdefghijklmnopqrstuvwxyz123456x:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_ending_with_a_dollar_sign() {
    assert_eq!(
        parse(b"abcdefghijklmnopqrstuvwxyz12345x$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names() {
    assert_eq!(
        parse(b"abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_ending_with_a_dollar_sign() {
    assert_eq!(
        parse(
            b"abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_initial_underscores() {
    assert_eq!(
        parse(b"_user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"_user"))]),
    );
}

#[test]
fn disallows_initial_dollar_signs() {
    assert_eq!(
        parse(b"$user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_hyphens() {
    assert_eq!(
        parse(b"-user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_numbers() {
    assert_eq!(
        parse(b"0user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"1user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"2user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"3user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"4user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"5user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"6user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"7user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"8user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"9user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_hash_signs() {
    assert_eq!(
        parse(b"#user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_names_with_special_characters_in_the_middle() {
    assert_eq!(
        parse(b"s!o@m#e%u^s&e*r:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_inner_hyphens() {
    assert_eq!(
        parse(b"some-user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"some-user"))]),
    );
}

#[test]
fn allows_trailing_hyphens() {
    assert_eq!(
        parse(b"user-:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user-"))]),
    );
}

#[test]
fn allows_inner_underscores() {
    assert_eq!(
        parse(b"some_user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"some_user"))]),
    );
}

#[test]
fn allows_trailing_underscores() {
    assert_eq!(
        parse(b"user_:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user_"))]),
    );
}

#[test]
fn detects_empty_names_after_empty_line() {
    assert_eq!(parse(b"\n"), IdTable::from_entries(&[]));
}

#[test]
fn allows_single_character_names_after_empty_line() {
    assert_eq!(
        parse(b"\na:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"a"))]),
    );
}

#[test]
fn allows_empty_passwords_after_empty_line() {
    assert_eq!(
        parse(b"\na::123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"a"))]),
    );
}

#[test]
fn disallows_usernames_not_followed_by_a_colon_after_empty_line() {
    assert_eq!(parse(b"\na"), IdTable::from_entries(&[]));
}

#[test]
fn disallows_empty_usernames_after_empty_line() {
    assert_eq!(
        parse(b"\n:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_break_after_password_after_empty_line() {
    assert_eq!(parse(b"\na:x\n"), IdTable::from_entries(&[]));
}

#[test]
fn disallows_empty_ids_after_empty_line() {
    assert_eq!(
        parse(b"\na:x::123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_break_before_id_after_empty_line() {
    assert_eq!(parse(b"\na:x:\n"), IdTable::from_entries(&[]));
}

#[test]
fn disallows_break_before_id_end_after_empty_line() {
    assert_eq!(parse(b"\na:x:1\n"), IdTable::from_entries(&[]));
}

#[test]
fn allows_break_after_id_end_after_empty_line() {
    assert_eq!(
        parse(b"\na:x:1:\n"),
        IdTable::from_entries(&[(1, name(b"a"))])
    );
}

#[test]
fn allows_small_alphanumeric_names_after_empty_line() {
    assert_eq!(
        parse(b"\nabc123:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abc123"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_ending_with_a_dollar_sign_after_empty_line() {
    assert_eq!(
        parse(b"\nabc123$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abc123$"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_after_empty_line() {
    assert_eq!(
        parse(b"\nabcdefghijklmnopqrstuvwxyz123456:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz123456"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_ending_with_a_dollar_sign_after_empty_line() {
    assert_eq!(
        parse(b"\nabcdefghijklmnopqrstuvwxyz12345$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz12345$"))]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_after_empty_line() {
    assert_eq!(
        parse(b"\nabcdefghijklmnopqrstuvwxyz123456x:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_empty_line() {
    assert_eq!(
        parse(b"\nabcdefghijklmnopqrstuvwxyz12345x$:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_after_empty_line() {
    assert_eq!(
        parse(
            b"\nabcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_empty_line() {
    assert_eq!(
        parse(
            b"\nabcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_initial_underscores_after_empty_line() {
    assert_eq!(
        parse(b"\n_user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"_user"))]),
    );
}

#[test]
fn disallows_initial_dollar_signs_after_empty_line() {
    assert_eq!(
        parse(b"\n$user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_hyphens_after_empty_line() {
    assert_eq!(
        parse(b"\n-user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_numbers_after_empty_line() {
    assert_eq!(
        parse(b"\n0user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n1user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n2user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n3user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n4user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n5user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n6user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n7user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n8user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"\n9user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_initial_hash_signs_after_empty_line() {
    assert_eq!(
        parse(b"\n#user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn disallows_names_with_special_characters_in_the_middle_after_empty_line() {
    assert_eq!(
        parse(b"\ns!o@m#e%u^s&e*r:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_inner_hyphens_after_empty_line() {
    assert_eq!(
        parse(b"\nsome-user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"some-user"))]),
    );
}

#[test]
fn allows_trailing_hyphens_after_empty_line() {
    assert_eq!(
        parse(b"\nuser-:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user-"))]),
    );
}

#[test]
fn allows_inner_underscores_after_empty_line() {
    assert_eq!(
        parse(b"\nsome_user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"some_user"))]),
    );
}

#[test]
fn allows_trailing_underscores_after_empty_line() {
    assert_eq!(
        parse(b"\nuser_:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user_"))]),
    );
}

#[test]
fn detects_empty_names_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn allows_single_character_names_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"a"))]),
    );
}

#[test]
fn allows_empty_passwords_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na::456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"a"))]),
    );
}

#[test]
fn disallows_usernames_not_followed_by_a_colon_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_empty_usernames_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_break_after_password_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x\n"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_empty_ids_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x::123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_break_before_id_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x:\n"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_break_before_id_end_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x:1\n"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn allows_break_after_id_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\na:x:1:\n"),
        IdTable::from_entries(&[(123, name(b"user")), (1, name(b"a"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nabc123:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"abc123"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_ending_with_a_dollar_sign_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nabc123$:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"abc123$"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_after_successful_line() {
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz123456:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz123456"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_ending_with_a_dollar_sign_after_successful_line() {
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz12345$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz12345$"))]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_after_successful_line() {
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz123456x:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_successful_line()
{
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz12345x$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_after_successful_line() {
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_successful_line() {
    assert_eq!(
        parse(
            b"user:x:123:123:::/sbin/nologin
abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn allows_initial_underscores_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n_user:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"_user"))]),
    );
}

#[test]
fn disallows_initial_dollar_signs_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n$user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_initial_hyphens_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n-user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_initial_numbers_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n0user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n1user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n2user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n3user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n4user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n5user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n6user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n7user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n8user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n9user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_initial_hash_signs_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\n#user:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn disallows_names_with_special_characters_in_the_middle_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\ns!o@m#e%u^s&e*r:x:123:123:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user"))]),
    );
}

#[test]
fn allows_inner_hyphens_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nsome-user:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"some-user"))]),
    );
}

#[test]
fn allows_trailing_hyphens_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nuser-:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"user-"))]),
    );
}

#[test]
fn allows_inner_underscores_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nsome_user:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"some_user"))]),
    );
}

#[test]
fn allows_trailing_underscores_after_successful_line() {
    assert_eq!(
        parse(b"user:x:123:123:::/sbin/nologin\nuser_:x:456:456:::/sbin/nologin"),
        IdTable::from_entries(&[(123, name(b"user")), (456, name(b"user_"))]),
    );
}

#[test]
fn detects_empty_names_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\n"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_single_character_names_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&\na:x:123:123:::/sbin/nologin"
        ),
        IdTable::from_entries(&[(123, name(b"a"))]),
    );
}

#[test]
fn allows_empty_passwords_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&\na::123:123:::/sbin/nologin"
        ),
        IdTable::from_entries(&[(123, name(b"a"))]),
    );
}

#[test]
fn disallows_usernames_not_followed_by_a_colon_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\na"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_empty_usernames_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&\n:x:123:123:::/sbin/nologin"
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_break_after_password_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\na:x\n"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_empty_ids_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
a:x::123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_break_before_id_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\na:x:\n"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_break_before_id_end_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\na:x:1\n"),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_break_after_id_end_after_dropped_line() {
    assert_eq!(
        parse(b"intentionally invalid line with spaces and !other# n0n$3nse&\na:x:1:\n"),
        IdTable::from_entries(&[(1, name(b"a"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abc123:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abc123"))]),
    );
}

#[test]
fn allows_small_alphanumeric_names_ending_with_a_dollar_sign_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abc123$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abc123$"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz123456:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz123456"))]),
    );
}

#[test]
fn allows_large_alphanumeric_names_ending_with_a_dollar_sign_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz12345$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"abcdefghijklmnopqrstuvwxyz12345$"))]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz123456x:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_slightly_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz12345x$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_far_too_large_alphanumeric_names_ending_with_a_dollar_sign_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvw$:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_initial_underscores_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
_user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"_user"))]),
    );
}

#[test]
fn disallows_initial_dollar_signs_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
$user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_initial_hyphens_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
-user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_initial_numbers_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
0user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
1user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
2user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
3user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
4user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
5user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
6user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
7user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
8user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
9user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_initial_hash_signs_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
#user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn disallows_names_with_special_characters_in_the_middle_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
s!o@m#e%u^s&e*r:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[]),
    );
}

#[test]
fn allows_inner_hyphens_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
some-user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"some-user"))]),
    );
}

#[test]
fn allows_trailing_hyphens_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
user-:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user-"))]),
    );
}

#[test]
fn allows_inner_underscores_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
some_user:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"some_user"))]),
    );
}

#[test]
fn allows_trailing_underscores_after_dropped_line() {
    assert_eq!(
        parse(
            b"intentionally invalid line with spaces and !other# n0n$3nse&
user_:x:123:123:::/sbin/nologin",
        ),
        IdTable::from_entries(&[(123, name(b"user_"))]),
    );
}

#[test]
fn works_for_unpadded_powers_of_10() {
    assert_eq!(
        parse(b"user:x:10:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:100:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:10000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:100000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:10000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:100000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000000, name(b"user"))]),
    );
}

#[test]
fn works_for_padded_powers_of_10() {
    assert_eq!(
        parse(b"user:x:0000000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000000010:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000000100:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000001000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000010000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000100000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0001000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0010000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0100000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000000, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000000, name(b"user"))]),
    );
}

#[test]
fn works_for_unpadded_powers_of_10_plus_1() {
    assert_eq!(
        parse(b"user:x:11:123:::/sbin/nologin"),
        IdTable::from_entries(&[(11, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:101:123:::/sbin/nologin"),
        IdTable::from_entries(&[(101, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:10001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:100001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:10000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:100000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000001, name(b"user"))]),
    );
}

#[test]
fn works_for_padded_powers_of_10_plus_1() {
    assert_eq!(
        parse(b"user:x:0000000011:123:::/sbin/nologin"),
        IdTable::from_entries(&[(11, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000000101:123:::/sbin/nologin"),
        IdTable::from_entries(&[(101, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000001001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000010001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0000100001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0001000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0010000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(10000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:0100000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(100000001, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:1000000001:123:::/sbin/nologin"),
        IdTable::from_entries(&[(1000000001, name(b"user"))]),
    );
}

#[test]
fn works_near_u32_representation_limit() {
    assert_eq!(
        parse(b"user:x:4294967290:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967290, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967291:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967291, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967292:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967292, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967293:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967293, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967294:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967294, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967295:123:::/sbin/nologin"),
        IdTable::from_entries(&[(4294967295, name(b"user"))]),
    );
    assert_eq!(
        parse(b"user:x:4294967296:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:4294967297:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:4294967298:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:4294967299:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn rejects_numbers_suffixed_with_hyphens() {
    assert_eq!(
        parse(b"user:x:0-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:10-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:100-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:1000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:10000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:100000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:1000000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:10000000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:100000000-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:429496729-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:4294967295-:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}

#[test]
fn rejects_numbers_prefixed_with_hyphens() {
    assert_eq!(
        parse(b"user:x:-0:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-10:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-100:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-1000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-10000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-100000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-1000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-10000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-100000000:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-429496729:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
    assert_eq!(
        parse(b"user:x:-4294967295:123:::/sbin/nologin"),
        IdTable::from_entries(&[])
    );
}
