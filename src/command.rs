use super::database::Database;
use super::database::Value;
use super::parser::Parser;

pub enum Response {
    Nil,
    Data(Vec<u8>),
    Error(String),
    Status(String),
}

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        match *self {
            Response::Nil => return b"$-1\r\n".to_vec(),
            Response::Data(ref d) => {
                return b"$".to_vec() + format!("{}\r\n", d.len()).as_bytes() + d + format!("\r\n").as_bytes();
            }
            Response::Error(ref d) => {
                return b"-".to_vec() + (*d).as_bytes() + format!("\r\n").as_bytes();
            }
            Response::Status(ref d) => {
                return b"+".to_vec() + format!("{}\r\n", d).as_bytes();
            }
        }
    }
}

macro_rules! validate {
    ($expr: expr, $err: expr) => (
        if !($expr) {
            return Response::Error($err.to_string());
        }
    )
}

macro_rules! try_validate {
    ($expr: expr, $err: expr) => ({
        let res = $expr;
        validate!(res.is_ok(), $err);
        res.unwrap()
    })
}

fn set(parser: &Parser, db: &mut Database) -> Response {
    validate!(parser.argc == 3, "Wrong number of parameters");
    let key = try_validate!(parser.get_vec(1), "Invalid key");
    let val = try_validate!(parser.get_vec(2), "Invalid value");
    db.get_or_create(&key).set(val);
    return Response::Status("OK".to_string());
}

fn append(parser: &Parser, db: &mut Database) -> Response {
    validate!(parser.argc == 3, "Wrong number of parameters");
    let key = try_validate!(parser.get_vec(1), "Invalid key");
    let val = try_validate!(parser.get_vec(2), "Invalid value");
    db.get_or_create(&key).set(val);
    return Response::Status("OK".to_string());
}

fn get(parser: &Parser, db: &mut Database) -> Response {
    validate!(parser.argc == 2, "Wrong number of parameters");
    let key = try_validate!(parser.get_vec(1), "Invalid key");
    let obj = db.get(&key);
    match obj {
        Some(value) => {
            match value {
                &Value::Data(ref data) => return Response::Data(data.clone()),
                &Value::Integer(ref int) => return Response::Data(format!("{}", int).into_bytes()),
                &Value::Nil => panic!("Should not have a nil"),
            }
        }
        None => return Response::Nil,
    }
}

fn ping(parser: &Parser, db: &mut Database) -> Response {
    #![allow(unused_variables)]
    validate!(parser.argc <= 2, "Wrong number of parameters");
    if parser.argc == 2 {
        return Response::Data(parser.get_vec(1).unwrap());
    }
    return Response::Data(b"PONG".to_vec());
}

pub fn command(parser: &Parser, db: &mut Database) -> Response {
    if parser.argc == 0 {
        return Response::Error("Not enough arguments".to_string());
    }
    let try_command = parser.get_str(0);
    if try_command.is_err() {
        return Response::Error("Invalid command".to_string());
    }
    match try_command.unwrap() {
        "set" => return set(parser, db),
        "append" => return append(parser, db),
        "get" => return get(parser, db),
        "ping" => return ping(parser, db),
        _ => return Response::Error("Unknown command".to_string()),
    };
}
