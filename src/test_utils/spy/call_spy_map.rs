use crate::prelude::*;

use std::collections::VecDeque;

#[derive(Debug)]
struct SpyState<K, I, O> {
    key: K,
    args: Vec<I>,
    results: VecDeque<O>,
}

pub struct CallSpyMap<K, I, O> {
    states: Mutex<Vec<SpyState<K, I, O>>>,
    name: &'static str,
}

impl<K: PartialEq, I, O> CallSpyMap<K, I, O> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            states: Mutex::new(Vec::new()),
            name,
        }
    }

    pub fn enqueue(&self, key: K, result: O) {
        let mut guard = self.states.lock().unwrap_or_else(|e| e.into_inner());

        for state in guard.iter_mut() {
            if state.key == key {
                state.results.push_back(result);
                return;
            }
        }

        guard.push(SpyState {
            key,
            args: Vec::new(),
            results: VecDeque::from([result]),
        });
    }

    #[must_use]
    pub fn call(&self, key: K, args: I) -> O
    where
        K: fmt::Debug,
    {
        let mut guard = self.states.lock().unwrap_or_else(|e| e.into_inner());

        for state in guard.iter_mut() {
            if state.key == key {
                if let Some(result) = state.results.pop_front() {
                    state.args.push(args);
                    return result;
                }
            }
        }

        panic!("No more `{}` calls expected with key {:?}", self.name, key);
    }

    #[track_caller]
    pub fn assert_no_calls_remaining(&self)
    where
        K: fmt::Debug,
        I: fmt::Debug,
        O: fmt::Debug,
    {
        use std::fmt::Write;

        let mut fail_pairs = String::new();
        let mut prefix = "";

        let guard = self.states.lock().unwrap_or_else(|e| e.into_inner());

        for state in guard.iter() {
            if !state.results.is_empty() {
                write!(&mut fail_pairs, "{}{:?} => {:?}", prefix, &state.key, &state.results)
                    .unwrap();
                prefix = ", ";
            }
        }
        
        drop(guard); // don't poison

        if !fail_pairs.is_empty() {
            panic!(
                "Unexpected calls remaining for `{}`: {{{}}}",
                self.name, fail_pairs
            );
        }
    }

    #[track_caller]
    #[allow(unused)]
    pub fn assert_calls(&self, expected: &[(K, I)])
    where
        I: fmt::Debug + PartialEq,
        K: fmt::Debug,
    {
        let mut expected_map = Vec::<(&K, Vec<&I>)>::new();

        'outer: for (key, value) in expected.iter() {
            for (k, v) in expected_map.iter_mut() {
                if *k == key {
                    v.push(value);
                    continue 'outer;
                }
            }
            expected_map.push((key, vec![value]));
        }

        fn states_equal<K: PartialEq, I: PartialEq, O>(
            expected_map: &[(&K, Vec<&I>)],
            states: &[SpyState<K, I, O>],
        ) -> bool {
            if expected_map.len() != states.len() {
                return false;
            }

            'outer: for state in states {
                for (key, expected) in expected_map {
                    if *key == &state.key {
                        if !state.args.iter().eq(expected.iter().copied()) {
                            return false;
                        }

                        continue 'outer;
                    }
                }

                return false;
            }

            true
        }

        let states = self.states.lock().unwrap_or_else(|e| e.into_inner());

        if !states_equal(&expected_map, &states) {
            struct ExpectedStates<'a, K, I>(&'a [(&'a K, Vec<&'a I>)]);

            impl<K: PartialEq + fmt::Debug, I: fmt::Debug> fmt::Debug for ExpectedStates<'_, K, I> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_map()
                        .entries(self.0.iter().map(|s| (&s.0, &s.1)))
                        .finish()
                }
            }

            struct ActualStates<'a, K, I, O>(&'a [SpyState<K, I, O>]);

            impl<K: PartialEq + fmt::Debug, I: fmt::Debug, O> fmt::Debug for ActualStates<'_, K, I, O> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_map()
                        .entries(self.0.iter().map(|s| (&s.key, &s.args)))
                        .finish()
                }
            }

            panic!(
                "Calls for `{}` do not match.\nExpected: {:?}\n  Actual: {:?}",
                self.name,
                ExpectedStates(&expected_map),
                ActualStates(&states)
            )
        }
    }
}

impl<K: PartialEq, I, O> CallSpyMap<K, I, io::Result<O>> {
    pub fn enqueue_io(&self, key: K, result: Result<O, libc::c_int>) {
        self.enqueue(key, result.map_err(Error::from_raw_os_error));
    }
}
