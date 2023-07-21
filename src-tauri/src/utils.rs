use std::{future::Future, time::Instant};

use log::trace;

pub async fn trace_elapsed_time<T: Future>(label: &str, f: impl FnOnce() -> T) -> T::Output {
    let start = Instant::now();
    let result = f().await;
    let elapsed = start.elapsed();

    trace!("{} took {:.2?}", label, elapsed);

    result
}

const ALPHABET_START: u8 = 'a' as u8 - 1;
const ALPHABET_END: u8 = 'z' as u8 + 1;

// Based on https://stackoverflow.com/a/38927158/3816975
pub fn get_midpoint_string(prev: &str, next: &str) -> String {
    assert!(prev.is_ascii());
    assert!(next.is_ascii());
    // assert!(prev != next);

    let prev = prev.as_bytes();
    let next = next.as_bytes();

    let mut pos = 0;

    let mut p: u8;
    let mut n: u8;

    loop {
        p = *prev.get(pos).unwrap_or(&ALPHABET_START);
        n = *next.get(pos).unwrap_or(&ALPHABET_END);

        pos += 1;

        if p != n {
            break;
        }
    }

    let mut midpoint: Vec<u8> = Vec::with_capacity(pos + 1);

    if pos > 0 {
        midpoint.extend_from_slice(&prev[..pos - 1]);
    }

    if p == ALPHABET_START {
        while n == 'a' as u8 {
            n = match next.get(pos) {
                Some(n) => {
                    pos += 1;
                    *n
                }
                None => ALPHABET_END,
            };
            midpoint.push('a' as u8);
        }

        if n == 'b' as u8 {
            midpoint.push('a' as u8);
            n = ALPHABET_END;
        }
    } else if p + 1 == n {
        midpoint.push(p);
        n = ALPHABET_END;
        loop {
            p = match prev.get(pos) {
                Some(p) => {
                    pos += 1;
                    *p
                }
                None => ALPHABET_START,
            };

            if p != 'z' as u8 {
                break;
            }

            midpoint.push('z' as u8);
        }
    }

    // basically ceil((p + n) / 2)
    midpoint.push((p + n) / 2 + ((p + n) % 2));

    String::from_utf8(midpoint).expect("Midpoint process generates valid utf8")
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_get_midpoint_string() {
        assert_eq!(get_midpoint_string("abcde", "abchi"), "abcf");
        assert_eq!(get_midpoint_string("abc", "abchi"), "abcd");

        insta::assert_yaml_snapshot!(get_midpoint_string("abc", "abchi"));
        insta::assert_yaml_snapshot!(get_midpoint_string("", ""));
        insta::assert_yaml_snapshot!(get_midpoint_string("abc", ""));
        insta::assert_yaml_snapshot!(get_midpoint_string("", "abc"));
    }

    #[test]
    fn random_test_get_midpoint_string() {
        let mut data: Vec<String> = vec![get_midpoint_string("", "")];

        for _ in 0..1000 {
            let insert_index = rand::thread_rng().gen_range(0..=data.len());
            data.insert(
                insert_index,
                get_midpoint_string(
                    if insert_index > 0 {
                        data.get(insert_index - 1)
                            .map(|x| x.as_str())
                            .unwrap_or_default()
                    } else {
                        ""
                    },
                    data.get(insert_index)
                        .map(|x| x.as_str())
                        .unwrap_or_default(),
                ),
            );
        }

        trace!("{:?}", data);

        for slice in data.windows(2) {
            assert!(slice[0] < slice[1]);
        }
    }
}
