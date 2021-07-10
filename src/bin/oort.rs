use macroquad::input::KeyCode;
use macroquad::{audio, camera, input, shapes, window};
use nalgebra::point;
use oort::{frame_timer, renderer, simulation};

#[macroquad::main("Oort")]
async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let status_div = document
        .get_element_by_id("status")
        .expect("should have a status div");
    status_div.set_inner_html("Hello from Rust");

    let mut sim = simulation::Simulation::new();
    let mut renderer = renderer::Renderer::new();
    let collision_sound = audio::load_sound("assets/collision.wav").await.unwrap();
    let mut zoom = 0.001;
    let mut camera_target = point![0.0, 0.0];
    let mut frame_timer: frame_timer::FrameTimer = Default::default();
    let mut paused = false;
    let mut finished = false;
    let mut single_steps = 0;

    let scenario = oort::scenario::load("asteroid");
    scenario.init(&mut sim);

    loop {
        let mut status_msgs: Vec<String> = Vec::new();

        frame_timer.start("frame");

        let camera_step = 0.01 / zoom;
        if input::is_key_down(KeyCode::W) {
            camera_target.y += camera_step;
        }
        if input::is_key_down(KeyCode::S) {
            camera_target.y -= camera_step;
        }
        if input::is_key_down(KeyCode::A) {
            camera_target.x -= camera_step;
        }
        if input::is_key_down(KeyCode::D) {
            camera_target.x += camera_step;
        }
        if input::is_key_down(KeyCode::Z) {
            zoom *= 0.99;
        }
        if input::is_key_down(KeyCode::X) {
            zoom *= 1.01;
        }
        if input::is_key_down(KeyCode::Q) | input::is_key_down(KeyCode::Escape) {
            break;
        }
        if input::is_key_pressed(KeyCode::U) {
            for name in frame_timer.get_names() {
                let (a, b, c) = frame_timer.get(name);
                println!("{}: {:.1}/{:.1}/{:.1} ms", name, a * 1e3, b * 1e3, c * 1e3);
            }
            println!(
                "Number of: ships={} bullets={}",
                sim.ships.iter().count(),
                sim.bullets.iter().count()
            );
        }
        if input::is_key_pressed(KeyCode::Space) {
            paused = !paused;
            single_steps = 0;
        }
        if input::is_key_pressed(KeyCode::N) {
            paused = true;
            single_steps += 1;
        }

        if !paused {
            if let Some(&ship_handle) = sim.ships.iter().next() {
                let force = 1e4;
                if input::is_key_down(KeyCode::Up) {
                    sim.ship_mut(ship_handle).thrust_main(force);
                }
                if input::is_key_down(KeyCode::Down) {
                    sim.ship_mut(ship_handle).thrust_main(-force);
                }
                if input::is_key_down(KeyCode::Left) {
                    if input::is_key_down(KeyCode::LeftShift) {
                        sim.ship_mut(ship_handle).thrust_lateral(force);
                    } else {
                        sim.ship_mut(ship_handle).thrust_angular(force);
                    }
                }
                if input::is_key_down(KeyCode::Right) {
                    if input::is_key_down(KeyCode::LeftShift) {
                        sim.ship_mut(ship_handle).thrust_lateral(-force);
                    } else {
                        sim.ship_mut(ship_handle).thrust_angular(-force);
                    }
                }
                if input::is_key_pressed(KeyCode::F) {
                    sim.ship_mut(ship_handle).fire_weapon();
                }
                if input::is_key_down(KeyCode::LeftShift) && input::is_key_down(KeyCode::F) {
                    sim.ship_mut(ship_handle).fire_weapon();
                }
                if input::is_key_down(KeyCode::LeftShift) && input::is_key_pressed(KeyCode::K) {
                    sim.ship_mut(ship_handle).explode();
                }
            }
        }

        if !finished && scenario.tick(&mut sim) == oort::scenario::Status::Finished {
            finished = true;
        }

        if !finished && (!paused || single_steps > 0) {
            frame_timer.start("simulate");
            sim.step();
            frame_timer.end("simulate");
            if single_steps > 0 {
                single_steps -= 1;
            }
        }

        frame_timer.start("render");
        renderer.render(camera_target, zoom, &sim);
        frame_timer.end("render");

        if sim.collided {
            sim.collided = false;
            audio::play_sound_once(collision_sound);
            println!("collided");
        }

        frame_timer.end("frame");

        camera::set_default_camera();
        {
            let (a, b, c) = frame_timer.get("frame");
            status_msgs.push(format!(
                "Frame time: {:.1}/{:.1}/{:.1} ms",
                a * 1e3,
                b * 1e3,
                c * 1e3
            ));
        }

        if paused {
            status_msgs.push("PAUSED".to_string());
        } else if finished {
            status_msgs.push("FINISHED".to_string());
        }

        status_div.set_inner_html(&status_msgs.join("; "));

        // HACK required by macroquad.
        shapes::draw_circle(0.0, 0.0, 1.0, macroquad::color::WHITE);

        window::next_frame().await
    }
}
