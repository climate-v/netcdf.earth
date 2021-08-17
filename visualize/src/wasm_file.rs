use web_sys::{Blob, FileReaderSync, XmlHttpRequest};
use std::io::{Seek, SeekFrom, Read};
use js_sys::ArrayBuffer;
use crate::log;
use wasm_bindgen::{JsCast, JsValue};

fn to_js_error(value: JsValue) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, value.as_string().unwrap())
}

trait ReaderAbstraction {
    fn read_slice(&self, start: i32, end: i32) -> std::io::Result<ArrayBuffer>;

    fn total_size(&self) -> u64;
}

struct FileReader {
    file_ref: Blob
}

impl ReaderAbstraction for FileReader {
    fn read_slice(&self, start: i32, end: i32) -> std::io::Result<ArrayBuffer> {
        let file_reader = FileReaderSync::new().unwrap();
        let result: Blob = match self.file_ref.slice_with_i32_and_i32(start, end) {
            Ok(blob) => blob,
            Err(e) => return Err(to_js_error(e))
        };

        match file_reader.read_as_array_buffer(&result) {
            Err(e) => return Err(to_js_error(e)),
            Ok(v) => Ok(v)
        }
    }

    fn total_size(&self) -> u64 {
        self.file_ref.size() as u64
    }
}

struct HttpReader {
    url: String,
    size: u64
}

impl ReaderAbstraction for HttpReader {
    fn read_slice(&self, start: i32, end: i32) -> std::io::Result<ArrayBuffer> {
        let request = XmlHttpRequest::new().unwrap();
        log(&format!("Requesting bytes from {} to {}", start, end));
        request.set_request_header("Content-Range", &format!("bytes {}-{}/*", start, end)).unwrap();
        request.set_request_header("Content-Length", &format!("{}", (end - start) + 1)).unwrap();
        request.open_with_async("GET", &self.url, false).unwrap();
        request.send().unwrap();

        let response_status = request.status().unwrap();
        if response_status >= 200 && response_status < 300 {
            let response_data = request.response().unwrap();
            log(&format!("Got response {:?}", response_data));
            Ok(response_data.dyn_into::<ArrayBuffer>().unwrap())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Got invalid status"))
        }
    }

    fn total_size(&self) -> u64 {
        self.size
    }
}

pub struct WasmFile {
    reader: Box<dyn ReaderAbstraction>,
    seek_pos: u64
}

impl WasmFile {
    pub fn new(file: Blob) -> Self {
        WasmFile {
            reader: Box::new(FileReader {
                file_ref: file
            }),
            seek_pos: 0
        }
    }

    pub fn new_remote(url: String, size: u64) -> Self {
        WasmFile {
            reader: Box::new(HttpReader {
                url,
                size
            }),
            seek_pos: 0
        }
    }

    fn file_size(&self) -> u64 {
        self.reader.total_size()
    }
}

impl Seek for WasmFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                self.seek_pos = offset % self.file_size();
            }
            SeekFrom::End(offset) => {
                self.seek_pos = ((self.file_size() as i64 + offset) % self.file_size() as i64) as u64;
            }
            SeekFrom::Current(offset) => {
                self.seek_pos = ((self.seek_pos as i64 + offset) % self.file_size() as i64) as u64;
            }
        }
        Ok(self.seek_pos)
    }
}

impl Read for WasmFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let end_position = u64::min(self.seek_pos + buf.len() as u64, self.file_size());
        let total_read = end_position - self.seek_pos;
        let array_buffer = self.reader.read_slice(self.seek_pos as i32, end_position as i32)?;
        let uarray = js_sys::Uint8Array::new(&array_buffer);
        uarray.copy_to(buf);
        self.seek_pos = end_position;
        Ok(total_read as usize)
    }
}