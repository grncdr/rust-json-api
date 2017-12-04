enum Message {
    Quit,
    Patch(DocumentId, Patch),
}

struct Doc<W: Write> {
    value: Value,
    writer: W,
    version: usize,
}

type DocMap<W: Write> = RwLock<HashMap<String, RwLock<Doc<W>>>>;

struct Database<W: Write> {
    docs: DocMap<W>,
}


fn worker(rx: Receiver) {
    loop {
        match rx.recv().unwrap() {
            Message::Quit => break,
            Message::Patch(id, patch, tx) => tx.send(try_patch(id, patch)),
        }
    }
}

fn try_patch(&mut self, id: &str, patch: Patch) -> Result<(), ManyErrors> {
    let read_map = try!(self.docs.read());
    let extant = read_map.get(id);
    if extant == None {
        let doc = RwLock::new(Doc {
            value: Value::Null,
            writer: OpenOptions::new()
                        .append(true)
                        .create(true)
                        .write(true)
                        .open(self.filename_for(id)),
            version: 0,
        });
        drop(read_map);
        let id = id.to_string();
        let write_map = self.docs.write();
        write_map.insert(id, doc);
    }
}
