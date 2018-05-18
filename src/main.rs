extern crate crossbeam;
extern crate gif;
extern crate sdl2;

mod animation;
mod render;

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use animation::Animation;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

struct NextFrame;

fn main() {
    let path = env::args_os().nth(1).expect("one arg");

    let sdl = sdl2::init().expect("sdl2::init");

    let event = sdl.event().expect("sdl2::event");

    event.register_custom_event::<NextFrame>()
        .expect("register_custom_event");

    let video = sdl.video().unwrap();

    let window = video.window("GIFTOPIA2038", 0, 0)
        .opengl()
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .build()
        .unwrap();

    sdl.mouse().show_cursor(false);

    let texture_creator = canvas.texture_creator();

    let anim = Animation::load_gif(&texture_creator, &PathBuf::from(path))
        .expect("Animation::load_gif");

    let viewport = canvas.viewport();
    let target_width = (viewport.height() * anim.rect.width()) / anim.rect.height();
    let center = Rect::new(((viewport.width() - target_width) / 2) as i32, 0, target_width, viewport.height());

    let timer = sdl.timer().unwrap();

    let terminate = AtomicBool::new(false);

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            while !terminate.load(Ordering::SeqCst) {
                event.push_custom_event(NextFrame)
                    .expect("push_custom_event");

                thread::sleep(Duration::from_millis(40));
            }
        });

        let mut frames = anim.frames.iter().cycle();

        let mut event_pump = sdl.event_pump().unwrap();

        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    terminate.store(true, Ordering::SeqCst);
                    break;
                }
                ref ev if ev.is_user_event() => {
                    if let Some(NextFrame) = ev.as_user_event_type() {
                        let (tex, tex_flip) = frames.next().expect("frames.next");
                        canvas.copy(tex, anim.rect, center);
                        canvas.copy(tex_flip, anim.rect, { let mut r = center.clone(); let x = r.x() - center.width() as i32; r.set_x(x); r });
                        canvas.copy(tex_flip, anim.rect, { let mut r = center.clone(); let x = r.x() + center.width() as i32; r.set_x(x); r });
                        canvas.present();
                    }
                }
                _ => {}
            }
        }
    });
}
