pub mod sounds;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioBuffer, AudioContext, Request, RequestInit, RequestMode, Response,
};

use crate::audio::sounds::Sounds;

#[wasm_bindgen]
pub struct Audio {
    audio_context: AudioContext,
    audio_buffers: Vec<AudioBuffer>,
}

#[wasm_bindgen]
impl Audio {
    pub async fn new(window: &web_sys::Window) -> Audio {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let audio_context = web_sys::AudioContext::new().unwrap();

        let mut audio_buffers: Vec<AudioBuffer> = Vec::new();

        let hostname = window.location().hostname().unwrap();
        let port = window.location().port().unwrap();

        for sound in Sounds::SOUNDS {
            let url = format!("http://{}:{}/{}",  hostname, port, sound);

            let request = Request::new_with_str_and_init(&url, &opts).unwrap();
            let resp = JsFuture::from(window.fetch_with_request(&request))
                .await
                .unwrap();
            let resp: Response = resp.dyn_into().unwrap();

            let array_buffer = resp.array_buffer().unwrap();
            let array_buffer = JsFuture::from(array_buffer).await.unwrap();
            let array_buffer: js_sys::ArrayBuffer = array_buffer.dyn_into().unwrap();

            let audio_buffer = audio_context.decode_audio_data(&array_buffer).unwrap();
            let audio_buffer = JsFuture::from(audio_buffer).await.unwrap();
            let audio_buffer: web_sys::AudioBuffer = audio_buffer.dyn_into().unwrap();

            audio_buffers.push(audio_buffer);
        }

        return Audio {
            audio_context,
            audio_buffers,
        };
    }

    pub async fn play(&self, sound_type: usize, x: f32, y: f32) {
        // Create audio nodes
        let audio_source = self.audio_context.create_buffer_source().unwrap();
        audio_source.set_buffer(Some(&self.audio_buffers[sound_type]));

        let panner_node = self.audio_context.create_panner().unwrap();

        let x = (x/-100.0) as f64;
        let y = (y/-100.0) as f64;
        panner_node.set_position(x, y, 5.0);

        let destination_node = self.audio_context.destination();

        // Connect audio nodes
        audio_source
            .connect_with_audio_node(&panner_node)
            .unwrap();

        panner_node
            .connect_with_audio_node(&destination_node)
            .unwrap();

        audio_source.start().expect("Unable to start");
    }
}
