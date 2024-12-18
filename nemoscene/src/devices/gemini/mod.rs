use crate::devices::{Device, Value};
use lazy_static::lazy_static;
use log::info;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use faster_whisper_rs::config::WhisperConfig;
use faster_whisper_rs::WhisperModel;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

lazy_static! {
    static ref WHISPER: WhisperContext =
        WhisperContext::new_from_buffer_with_params(include_bytes!("ggml_tiny.bin"), WhisperContextParameters::default()).unwrap();
    static ref FASTER_WHISPER: WhisperModel = WhisperModel::new("tiny_en".to_string(), "cpu".to_string(), "int8".to_string(), WhisperConfig::default()).unwrap();
}

pub struct Gemini;

const RECORD_ON_SOUND: &[u8] = include_bytes!("record_on.raw");
const RECORD_OFF_SOUND: &[u8] = include_bytes!("record_off.raw");

impl Gemini {
    pub fn new() -> Gemini {
        Gemini
    }
}

impl Device for Gemini {
    fn name(&self) -> &str {
        "Gemini"
    }
    fn command(
        &mut self,
        command: &str,
        parameters: &BTreeMap<String, Value>,
    ) -> anyhow::Result<Value> {
        match command {
            "voice" => {
                thread::spawn(move || {
                    let args = ["-", "--rate", "16000", "--format", "s16", "--channels", "1"];
                    let mut recorder = Command::new("timeout")
                        .arg("30")
                        .arg("pw-record")
                        .args(args)
                        .env("XDG_RUNTIME_DIR", "/run/user/1000")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap();

                    let mut cmd = Command::new("pw-play")
                        .arg("-")
                        .env("XDG_RUNTIME_DIR", "/run/user/1000")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap();
                    let stdin = cmd.stdin.as_mut().unwrap();
                    stdin.write_all(RECORD_ON_SOUND).unwrap();

                    let mut recording = Vec::new();
                    {
                        let stdout = recorder.stdout.take().unwrap();
                        let mut stdout_reader = BufReader::new(stdout);
                        let frame_size = 1024;
                        let threshold = 20000;
                        let sample_size = 2;
                        let discard_first = 5;
                        let mut frames = 0;
                        let mut silence = 0;
                        let mut silence_frames = 40;
                        loop {
                            let mut buffer = vec![0; frame_size * sample_size];
                            let bytes_read = stdout_reader.read(&mut buffer).unwrap();
                            if bytes_read == 0 {
                                break;
                            }
                            frames += 1;
                            if frames < discard_first {
                                continue;
                            }
                            let mut energy = 0;
                            for i in 0..frame_size {
                                let sample = i16::from_le_bytes(
                                    buffer[i * sample_size..(i + 1) * sample_size]
                                        .try_into()
                                        .unwrap(),
                                );
                                recording.push(sample);
                                if let Some(e) = (sample as u64).checked_pow(2) {
                                    energy += e;
                                }
                            }
                            if energy < threshold {
                                silence += 1;
                            } else {
                                silence = 0;
                            }
                            if silence >= silence_frames {
                                info!("Silence for {} frames, done recording", silence_frames);
                                break;
                            }
                        }
                    }
                    recorder.kill().unwrap();

                    info!("Prompt: {:?}", Self::audio_to_text(recording.as_slice()));
                });
                /*cmd = Command::new("pw-play").arg("-").env("XDG_RUNTIME_DIR", "/run/user/1000").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
                let stdin = cmd.stdin.as_mut().unwrap();
                stdin.write_all(RECORD_OFF_SOUND)?;
                cmd.wait()?;*/

                Ok(Value::Bool(true))
            }
            _ => Ok(Value::Bool(false)),
        }
    }
}

impl Gemini {
    fn audio_to_text(audio_data: &[i16]) -> anyhow::Result<String> {
        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        //let audio_data = wav_io::convert_samples_i16_to_f32(&audio_data.to_vec());
        let filename = format!(
            "/tmp/voice{}.wav",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
        );
        let head = wav_io::new_header(16000, 16, false, true);
        let mut file_out = File::create(&filename)?;
        wav_io::write_to_file(
            &mut file_out,
            &head,
            &wav_io::convert_samples_i16_to_f32(&audio_data.to_vec()),
        )?;

        let tr = FASTER_WHISPER.transcribe(filename).unwrap().to_string();
        return Ok(tr);
/*
        let mut state = WHISPER.create_state()?;
        state.full(params, &audio_data[..])?;

        let num_segments = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            text += segment.as_str();
        }
        Ok(text)*/
        /* let filename = format!(
            "/tmp/voice{}.wav",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
        );
        let head = wav_io::new_header(16000, 16, false, true);
        let mut file_out = File::create(&filename)?;
        wav_io::write_to_file(
            &mut file_out,
            &head,
            &wav_io::convert_samples_i16_to_f32(&audio_data.to_vec()),
        )?;

        let client = Client::new();
        let url = format!(
            "https://speech.googleapis.com/v1/speech:recognize?key={}",
            "awesome-key"
        );

        let form = reqwest::blocking::multipart::Form::new().file("audio_file", &filename)?;

        let response = client
            .post(url)
            .header("Content-Type", "multipart/form-data")
            .multipart(form)
            .send()?;

        Ok(format!("{:?}", response.text()))*/
    }
}
