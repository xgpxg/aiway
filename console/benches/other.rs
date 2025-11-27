use criterion::{Criterion, criterion_group, criterion_main};

fn test_split_1() {
    let s = "hello world, today is a nice day";
    let mut splits = s.split(" ");

    while let Some(item) = splits.next() {
        let _ = item.to_string();
    }
    // let _ = splits.next().unwrap().to_string();
    // let _ = splits.next().unwrap().to_string();
}

fn test_split_2() {
    let s = "hello world, today is a nice day";
    let splits = s.split(" ").collect::<Vec<_>>();

    for split in splits.iter() {
        let _ = split.to_string();
    }
    // let _ = splits[0].to_string();
    // let _ = splits[1].to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test iter 1", |b| b.iter(|| test_split_1()));
    c.bench_function("test iter 2", |b| b.iter(|| test_split_2()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
