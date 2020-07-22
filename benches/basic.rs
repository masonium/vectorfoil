use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vectorfoil::Renderer;
use nalgebra_glm as glm;
use glm::{vec3, look_at, perspective};

fn basic_benchmark(c: &mut Criterion) {
    let view = look_at(&vec3(0.0, 0.0, 2.0),
		       &vec3(0.0, 0.0, 0.0),
		       &vec3(0.0, 1.0, 0.0));
    let proj = perspective(1.0, std::f64::consts::FRAC_PI_2, 1.0, 9.0);
    let mut renderer = Renderer::new(&(proj * view));

    renderer.add_tri(&vec3(-1.0, -1.0, 1.0),
		     &vec3(0.5, 0.0, 1.0),
		     &vec3(-1.0, 1.0, 1.0));
    renderer.add_tri(&vec3(-0.5, 0.0, -1.0),
		     &vec3(1.0, -1.0, -1.0),
		     &vec3(1.0, 1.0, -1.0));

    c.bench_function("two triangle", |b| b.iter(|| black_box(&renderer).render()));
}

criterion_group!(benches, basic_benchmark);
criterion_main!(benches);
