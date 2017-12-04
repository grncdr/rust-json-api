use std::fs::File;

pub struct MultiLog {
    writer_thread: usize,
}

enum Message {
    Write(String, &[u8], SyncSender<()>),
    Exit
}

pub impl MultiLog {
    fn new(log_dir: String) -> MultiLog {
        let mut ml = MultiLog { };
        ml
    }

    fn write(&mut self, id: String, bytes: &[u8]) -> Result<(), Blah> {
        let (rx, tx2) = sync_channel(0);
        try!(self.tx.send((id, bytes, tx2)));
        rx.recv()
    }

    fn writer(&mut self, id: String) -> Result<ThreadLogger> {
    }

    fn get_channel(&mut self, id: String) -> {
        let (rx, tx) = channel();
        ml.writer_thread = thread::spawn(writer_loop);
    }
}

pub struct ThreadWriter {
    filename: Path,
    file: File
}

impl ThreadWriter {
    fn create(filename: AsRef<Path>) -> ::std::io::Result<ThreadWriter> {
        OpenOptions.new().read(true).write(true)
    }
}

impl Write for ThreadWriter {
    fn write 
}

fn writer_loop (filename: &str, rx: Receiver<Message>) {
    loop {
        match rx.recv().unwrap() {
            Message::Write(bytes, reply_ch) => {
                if !files.contains_key(id) {
                    let f = File::open(Path::join(log_dir, id), Write); // or something
                    files.insert(id, f);
                }
                let f = files.get_mut(id);
                reply_ch.send(f.write_all(bytes));
            },

            Message::Exit => break
        };
    };
}

/*
   // this might be useful if I need lots of these receive loops
macro_rules! recv_loop {
    ($message_type:ident, $ch:expr) { $pattern:expr => $body:expr... } => {
        loop {
            match $ch.recv().unwrap() {
                $message_type::$pattern => $body...,
                $message_type::Exit => break

            }
        }
    }
}
*/
