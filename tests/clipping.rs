use glm::{look_at, perspective, vec3};
use nalgebra_glm as glm;
use vectorfoil::Renderer;

fn renderer() -> Renderer {
    let view = look_at(
        &vec3(0.0, 0.0, 5.0),
        &vec3(0.0, 0.0, 0.0),
        &vec3(0.0, 1.0, 0.0),
    );
    let proj = perspective(1.0, std::f64::consts::FRAC_PI_2, 0.1, 10.0);
    Renderer::new(&(proj * view))
}

#[test]
fn clip_point_neg_x() {
    let mut r = renderer();
    r.add_point(vec3(-10.0, 0.0, 0.0));
    assert!(r.render().is_empty());

    let mut r = renderer();
    r.add_point(vec3(-1.0, 0.0, 0.0));
    assert!(!r.render().is_empty());
}

#[test]
fn clip_point_pos_x() {
    let mut r = renderer();
    r.add_point(vec3(10.0, 0.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_point_neg_y() {
    let mut r = renderer();
    r.add_point(vec3(0.0, -10.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_point_pos_y() {
    let mut r = renderer();
    r.add_point(vec3(0.0, 10.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_point_neg_z() {
    let mut r = renderer();
    r.add_point(vec3(0.0, 0.0, 6.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_point_pos_z() {
    let mut r = renderer();
    r.add_point(vec3(0.0, 0.0, -6.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_neg_x() {
    let mut r = renderer();
    r.add_line(vec3(-10.0, 0.0, 0.0), vec3(-9.0, 0.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_pos_x() {
    let mut r = renderer();
    r.add_line(vec3(10.0, 0.0, 0.0), vec3(9.0, 0.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_neg_y() {
    let mut r = renderer();
    r.add_line(vec3(0.0, -10.0, 0.0), vec3(0.0, -9.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_pos_y() {
    let mut r = renderer();
    r.add_line(vec3(0.0, 10.0, 0.0), vec3(0.0, 9.0, 0.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_neg_z() {
    let mut r = renderer();
    r.add_line(vec3(0.0, 0.0, -10.0), vec3(0.0, 0.0, -9.0));
    assert!(r.render().is_empty());
}

#[test]
fn clip_line_pos_z() {
    let mut r = renderer();
    r.add_line(vec3(0.0, 0.0, 10.0), vec3(0.0, 0.0, 9.0));
    assert!(r.render().is_empty());
}
