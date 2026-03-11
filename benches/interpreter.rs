use std::{hint::black_box, io};

use brainfuck_rs::Brain;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const HELLO_WORLD: &str =
    "++++++++++[>+>+++>+++++++>++++++++++<<<<-]>>>++.>+.+++++++..+++.<<++.>+++++++++++++++.>.+++.------.--------.";
const NESTED_LOOPS: &str = "+++++[>++++++[>++<-]<-]>>+++++.";
const TAPE_WALK: &str =
    "++++++++++[>+>+<<-]>>[<<+>>-]<<<[>>>+<<<-]>>>++++++++++++++++++++++++++++++++++++++++++++++++.";

fn benchmark_programs(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter");

    for (name, program) in [
        ("hello_world", HELLO_WORLD),
        ("nested_loops", NESTED_LOOPS),
        ("tape_walk", TAPE_WALK),
    ] {
        group.bench_with_input(BenchmarkId::new("run", name), &program, |b, program| {
            b.iter(|| {
                let mut brain = Brain::new(&black_box(*program));
                brain
                    .run_with_output(&mut io::sink())
                    .expect("benchmark program should run");
                black_box(brain.output_string());
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_programs);
criterion_main!(benches);
