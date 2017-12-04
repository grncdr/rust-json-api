# Database design

The database is at it's core a k/v document store, where each document is stored
on disk as a series of JSON patches applied to `null`.

## Thread pools

### HTTP Request Thread Pool

Each request uses at most 1 doc at a time. If each doc can be used by multiple concurrent requests then reads against the doc itself should not be single-threaded. (or we serialize all requests)

Therefore read & write paths *must* be separated and reads can be handled directly on the request thread directly.

### DB Writer Thread Pool

The database maintains an internal threadpool where each thread is feeding off a queue. Each thread loops like so:

1. Get message
2. If message is `Quit` bail out
3. Otherwise the message should be a (DocumentId, Patch, Tx) tuple
4. Send the result of calling `try_patch` with the doc id & patch on Tx


`try_patch`
5. Serialize the patch for step 6.2

5. Fetch the document lock from the shared hashmap
5.1. If not found:
5.1.1. grab write lock on shared hashmap 
5.1.2. insert RwLock::new(Value::Null) into the map
5.1.3. (drop write lock)
5.1.4. grab write lock on doc
5.1.5. load doc from disk

6. With a *write* lock on the doc (that may have been acquired in 5.1.4)
6.1. Apply the patch to doc.value creating a new Value
6.2. Write the patch to the log file
6.3. Swap the value in doc
6.4. (write lock dropped)

7. Send Ok(()) on Tx

## Types

enum Message {
  Quit,
  Patch(DocumentId, Patch),
}

struct Doc <W: Write> {
  value: Value,
  writer: W,
  version: usize,
}

struct Database <W: Write> {
  docs: RwLock<HashMap<String, RwLock<Doc<W>>>>
}
