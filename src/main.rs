use std::process;

use gstreamer::prelude::*;

#[cfg(not(target_os = "macos"))]
fn run<T, F: FnOnce() -> T + Send + 'static>(main: F) -> T
where
    T: Send + 'static,
{
    main()
}

fn build_pipeline() {
    let pipeline_str = "videotestsrc ! autovideosink";

    gstreamer::init().unwrap();

    let mut context = gstreamer::ParseContext::new();

    let pipeline = match gstreamer::parse_launch_full(
        pipeline_str,
        Some(&mut context),
        gstreamer::ParseFlags::empty(),
    ) {
        Ok(pipeline) => pipeline,
        Err(err) => {
            if let Some(gstreamer::ParseError::NoSuchElement) = err.kind::<gstreamer::ParseError>()
            {
                println!("Missing element(s): {:?}", context.missing_elements());
            } else {
                println!("Failed to parse pipeline: {err}");
            }

            process::exit(-1)
        }
    };

    let bus = pipeline.bus().unwrap();

    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Unable to set the pipeline to the playing state");

    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                break;
            }
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }

    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the null state");
}

fn main() {
    run(build_pipeline);
}
