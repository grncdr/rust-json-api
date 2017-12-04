use patch_helpers::prefix_patch_paths;

use std::collections::HashMap;
use std::io::Write;
use std::fs::File;
use std::sync::{RwLock, PoisonError};
use unicase::UniCase;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::server::{Handler, Server, Listening, Request, Response};
use hyper::uri::RequestUri;
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowMethods, AccessControlAllowHeaders,
                    ContentType};

use serde_json;
use serde_json::Value;
use json_patch;
use json_patch::{Op, Patch};

use database::{Database, DbError};
use shared_value::SharedValue;

struct App(Database<File>);

#[derive(Debug)]
struct GlobalJsonPointer<'a> {
    doc_id: &'a str,
    pointer: Vec<&'a str>,
}

#[derive(Debug)]
pub enum ApiError {
    BadUri,
    DocumentDoesNotExist,
    PathDoesNotExist,
    JsonError(serde_json::Error),
    InvalidPatchError(json_patch::InvalidPatchError),
    PatchFailedError(json_patch::PatchError),
    DbError(DbError),
}

wrap_error!(DbError, ApiError::DbError);
wrap_error!(json_patch::InvalidPatchError, ApiError::InvalidPatchError);
wrap_error!(serde_json::Error, ApiError::JsonError);
wrap_error!(json_patch::PatchError, ApiError::PatchFailedError);

struct Reply(StatusCode, String);

impl From<Value> for Reply {
    fn from(v: Value) -> Reply {
        Reply(StatusCode::Ok, format!("{:?}", v))
    }
}

impl From<Result<Reply, ApiError>> for Reply {
    fn from(result: Result<Reply, ApiError>) -> Reply {
        match result {
            Ok(v) => v,
            Err(e) => e.into(),
        }
    }
}

impl From<ApiError> for Reply {
    fn from(err: ApiError) -> Reply {
        let (code, message) = match err {
            ApiError::JsonError(e) => (StatusCode::BadRequest, format!("{:?}", e)),
            ApiError::InvalidPatchError(e) => (StatusCode::BadRequest, format!("{:?}", e)),
            ApiError::BadUri => (StatusCode::BadRequest,
                                 "URI must be utf8 with at least one path component".into()),
            ApiError::DbError(e) => (StatusCode::InternalServerError, format!("{:?}", e)),
            ApiError::DocumentDoesNotExist => (StatusCode::NotFound, "no such document".into()),
            ApiError::PathDoesNotExist => (StatusCode::NotFound, "path does not exist".into()),
            ApiError::PatchFailedError(_) => (StatusCode::BadRequest,
                                              "patch could not be applied".into()),
        };

        Reply(code, format!(r#"{{"message":"{}"}}"#, message))
    }
}

impl <'a>From<(StatusCode, &'a str)> for Reply {
    fn from(tuple: (StatusCode, &str)) -> Reply {
        Reply(tuple.0, tuple.1.into())
    }
}


fn parse_uri<'a>(uri: &'a RequestUri) -> Result<GlobalJsonPointer<'a>, ApiError> {
    match uri {
        &RequestUri::AbsolutePath(ref string_path) => {
            let mut parts = string_path.split("/").skip(1);
            let doc_id = try!(parts.next().ok_or(ApiError::BadUri));
            Ok(GlobalJsonPointer {
                doc_id: doc_id,
                pointer: parts.collect(),
            })
        }
        _ => Err(ApiError::BadUri),
    }
}

fn parse_patch(req: Request) -> Result<Patch, ApiError> {
    match req.method {
        Method::Get => unreachable!(),
        Method::Patch => {
            let value = try!(serde_json::from_reader(req));
            Patch::from_value(value).map_err(|e| e.into())
        }
        Method::Put => {
            let value = try!(serde_json::from_reader(req));
            Ok(Patch { ops: vec![Op::Add(vec![], value)] })
        }
        Method::Delete => {
            Ok(Patch { ops: vec![Op::Remove(vec![])] })
        }
        _ => Err(ApiError::BadUri),
    }
}

pub fn start(addr: &str) -> Listening {
    let db = Database::open("./logs").unwrap();
    let app = App(db);
    Server::http(addr).unwrap().handle(app).unwrap()
}

impl Handler for App {
    fn handle(&self, req: Request, mut res: Response) {
        {
            let headers = res.headers_mut();
            headers.set(AccessControlAllowOrigin::Any);
            headers.set(AccessControlAllowMethods(vec![Method::Get, Method::Patch]));
            headers.set(AccessControlAllowHeaders(vec![UniCase("content-type".into()),

                                                       UniCase("authorization".into())]));
            headers.set(ContentType(mime!(Application / Json)));
        }

        let reply: Reply = self.try_request(req).into();

        {
            let mut status = res.status_mut();
            *status = reply.0;
        }
        match res.start().unwrap().write_all(reply.1.as_bytes()) {
            Ok(_) => (),
            Err(err) => {
                println!("Error writing response: {}", err);
            }
        }
    }
}

impl App {
    fn try_request(&self, req: Request) -> Result<Reply, ApiError> {
        let uri = req.uri.clone();
        let p = try!(parse_uri(&uri));

        if req.method == Method::Get {
            return self.0
                       .find_in_doc(p.doc_id, &p.pointer)
                       .map(|v| v.into())
                       .map_err(|e| e.into());
        }

        let patch = try!(parse_patch(req));
        match self.0.patch_doc(p.doc_id, patch, &p.pointer) {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(e.into()),
        }
    }
}
