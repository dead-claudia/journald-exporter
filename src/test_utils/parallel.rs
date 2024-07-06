pub fn parallel_for<T>(
    mut range: impl Iterator<Item = T> + Clone + Send,
    f: impl Fn(T) + Send + Clone,
) {
    let shards = std::thread::available_parallelism().unwrap().get();

    std::thread::scope(move |s| {
        for _ in 0..shards {
            let f = f.clone();
            let mut source = range.clone();
            range.next();

            s.spawn(move || {
                while let Some(i) = source.nth(shards) {
                    f(i);
                }
            });
        }
    })
}
