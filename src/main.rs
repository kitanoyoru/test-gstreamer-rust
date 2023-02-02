use anyhow::Error;
use gstreamer::prelude::*;

#[cfg(not(target_os = "macos"))]
fn run<T, F: FnOnce() -> T + Send + 'static>(main: F) -> T
where
    T: Send + 'static,
{
    main()
}

fn build_pipeline() -> Result<gstreamer::Pipeline, Error> {
    gstreamer::init()?;

    let pipeline = gstreamer::Pipeline::default();

    let src = gstreamer::ElementFactory::make("videotestsrc")
        .name("test_src")
        .build()?;
    let sink = gstreamer::ElementFactory::make("autovideosink")
        .name("test_sink")
        .build()?;

    pipeline.add_many(&[&src, &sink])?;
    gstreamer::Element::link_many(&[&src, &sink])?;

    Ok(pipeline)
}

fn main_loop(pipeline: gstreamer::Pipeline) -> Result<(), Error> {
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Unable to set the pipeline to the playing state");

    let bus = pipeline.bus().unwrap();

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
            MessageView::StateChanged(state) => {
                if state.src().map(|s| s == pipeline).unwrap_or(false) {
                    let new_state = state.current();
                    let old_state = state.old();

                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        old_state, new_state
                    );

                    let element = pipeline.by_name("test_sink").unwrap();

                    print_pad_capabilities(&element, "sink")
                }
            }
            _ => (),
        }
    }

    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the null state");

    Ok(())
}

fn print_pad_capabilities(element: &gstreamer::Element, pad_name: &str) {
    let pad = element.static_pad(pad_name).expect("Could not retrice pad");
    println!("Caps for the {} pad:", pad_name);
    let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
    print_caps(&caps, "     ");
}

fn print_caps(caps: &gstreamer::Caps, prefix: &str) {
    if caps.is_any() {
        println!("{}ANY", prefix);
        return;
    }

    if caps.is_empty() {
        println!("{}EMPTY", prefix);
        return;
    }

    for structure in caps.iter() {
        println!("{}{}", prefix, structure.name());
        for (field, value) in structure.iter() {
            println!(
                "{}  {}:{}",
                prefix,
                field,
                value.serialize().unwrap().as_str()
            );
        }
    }
}

fn example_main() {
    match build_pipeline().and_then(main_loop) {
        Ok(r) => r,
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() {
    run(example_main)
}
