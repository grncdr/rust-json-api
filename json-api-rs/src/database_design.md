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
1. Serialize the patch for step 3.2

2. Fetch the document lock from the shared hashmap
2.1. If not found:
2.1.1. grab write lock on shared hashmap 
2.1.2. insert RwLock::new(Value::Null) into the map
2.1.3. (drop write lock)
2.1.4. grab write lock on doc
2.1.5. load doc from disk

3. With a *write* lock on the doc (that may have been acquired in 2.1.4)
3.1. Apply the patch to doc.value creating a new Value
3.2. Write the patch to the log file
3.3. Swap the value in doc
3.4. (write lock dropped)

4. Send Ok(()) on Tx

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
