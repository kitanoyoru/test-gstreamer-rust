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
        .property_from_str("pattern", "ball")
        .build()?;
    let sink = gstreamer::ElementFactory::make("autovideosink")
        .name("test_sink")
        .build()?;

    let caps = gstreamer_video::VideoCapsBuilder::new()
        .width(800)
        .height(800)
        .framerate((60, 1).into())
        .build();

    let capsfilter = gstreamer::ElementFactory::make("capsfilter")
        .property("caps", &caps)
        .build()?;

    pipeline.add_many(&[&src, &capsfilter, &sink])?;
    gstreamer::Element::link_many(&[&src, &capsfilter, &sink])?;

    let srcpad = src.static_pad("src").unwrap();

    srcpad.add_probe(
        gstreamer::PadProbeType::DATA_DOWNSTREAM,
        move |_, probe_info| {
            match probe_info.data {
                Some(gstreamer::PadProbeData::Buffer(ref data)) => {
                    println!("src: {}", data.size());
                }
                _ => (),
            }

            gstreamer::PadProbeReturn::Ok
        },
    );

    let sinkpad = sink.static_pad("sink").unwrap();

    sinkpad.add_probe(
        gstreamer::PadProbeType::DATA_DOWNSTREAM,
        move |_, probe_info| {
            match probe_info.data {
                Some(gstreamer::PadProbeData::Buffer(ref data)) => {
                    println!("sink: {}", data.size());
                }
                _ => (),
            }

            gstreamer::PadProbeReturn::Ok
        },
    );

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
            _ => (),
        }
    }

    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the null state");

    Ok(())
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
