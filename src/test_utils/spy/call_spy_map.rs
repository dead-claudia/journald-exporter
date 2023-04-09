use crate::prelude::*;

use std::collections::hash_map;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;

#[derive(Debug)]
struct SpyState<I, O> {
    args: Vec<I>,
    results: VecDeque<O>,
}

pub struct CallSpyMap<K, I, O> {
    states: Mutex<Option<HashMap<K, SpyState<I, O>>>>,
    name: &'static str,
}

impl<K: Eq + Hash, I, O> CallSpyMap<K, I, O> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            states: Mutex::new(None),
            name,
        }
    }

    pub fn enqueue(&self, key: K, result: O) {
        let mut guard = self.states.lock().unwrap_or_else(|e| e.into_inner());

        let state = match guard.get_or_insert_with(HashMap::new).entry(key) {
            hash_map::Entry::Occupied(o) => o.into_mut(),
            hash_map::Entry::Vacant(v) => v.insert(SpyState {
                args: Vec::new(),
                results: VecDeque::new(),
            }),
        };

        state.results.push_back(result);
    }

    #[must_use]
    pub fn call(&self, key: K, args: I) -> O
    where
        K: fmt::Debug,
    {
        let mut guard = self.states.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(map) = &mut *guard {
            if let Some(state) = map.get_mut(&key) {
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
        let mut results = HashMap::new();

        let guard = self.states.lock().unwrap_or_else(|e| e.into_inner());
        for (key, state) in guard.iter().flatten() {
            if !state.results.is_empty() {
                results.insert(key, &state.results);
            }
        }

        if !results.is_empty() {
            panic!(
                "Unexpected calls remaining for `{}`: {:?}",
                self.name, results
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
        let mut expected_map: HashMap<&K, Vec<&I>> = HashMap::new();

        for (key, value) in expected {
            let vec = match expected_map.entry(key) {
                hash_map::Entry::Occupied(o) => o.into_mut(),
                hash_map::Entry::Vacant(v) => v.insert(Vec::new()),
            };
            vec.push(value);
        }

        fn args_equal<I: PartialEq>(expected: Option<&[&I]>, actual: &[I]) -> bool {
            match expected {
                Some(expected) => {
                    let mut expected = expected.iter();
                    let mut actual = actual.iter();
                    loop {
                        match (expected.next(), actual.next()) {
                            (None, None) => break true,
                            (Some(&a), Some(b)) if a == b => {}
                            (_, _) => break false,
                        }
                    }
                }
                _ => false,
            }
        }

        fn states_equal<K: Eq + Hash, I: PartialEq, O>(
            expected_map: &HashMap<&K, Vec<&I>>,
            states: &Option<HashMap<K, SpyState<I, O>>>,
        ) -> bool {
            let Some(states) = states.as_ref() else {
                return expected_map.is_empty();
            };

            if expected_map.len() != states.len() {
                return false;
            }

            for (key, state) in states.iter() {
                let Some(expected) = expected_map.get(key) else {
                    return false;
                };

                if expected.len() != state.args.len() {
                    return false;
                }

                for (a, b) in expected.iter().zip(state.args.iter()) {
                    if a != &b {
                        return false;
                    }
                }
            }

            true
        }

        let states = self.states.lock().unwrap_or_else(|e| e.into_inner());

        if !states_equal(&expected_map, &states) {
            struct ActualStates<'a, K, I, O>(&'a Option<HashMap<K, SpyState<I, O>>>);

            impl<K: Eq + Hash + fmt::Debug, I: fmt::Debug, O> fmt::Debug for ActualStates<'_, K, I, O> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_map()
                        .entries(self.0.iter().flatten().map(|(k, v)| (k, &v.args)))
                        .finish()
                }
            }

            panic!(
                "Calls for `{}` do not match.\nExpected: {:?}\n  Actual: {:?}",
                self.name,
                expected_map,
                ActualStates(&*states)
            )
        }
    }
}

impl<K: Eq + Hash, I, O> CallSpyMap<K, I, io::Result<O>> {
    pub fn enqueue_io(&self, key: K, result: Result<O, libc::c_int>) {
        self.enqueue(key, result.map_err(Error::from_raw_os_error));
    }
}
