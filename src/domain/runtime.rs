use serde::{
    Serialize,
    Deserialize,
    Serializer,
    Deserializer,
    de::Error,
};

use std::str::FromStr;

#[derive(sqlx::Type, Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
#[sqlx(transparent)]
pub struct RunTime(i32);


impl RunTime {
    pub fn is_zero(&self)-> bool {
           self.0 == 0
    }
}

impl AsRef<i32> for RunTime {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl std::convert::From<i32> for RunTime {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for RunTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} mins", self.0)
    }
}

impl FromStr for RunTime {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(' ').collect::<Vec<&str>>();
        if v.len() != 2 || v[1] != "mins" {
            return Err("invalid runtime format");
        }
        Ok(Self(i32::from_str(v[0]).map_err(|_| "invalid runtime format")?))
    }
}


impl Serialize for RunTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&format_args!("{} mins", self.0))
    }
}

impl<'de> Deserialize<'de> for RunTime {
    fn deserialize<D>(deserializer: D) -> Result<RunTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        RunTime::from_str(s).map_err(Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct W {
    a: i32,
    b: String,
    c: RunTime,
}

/*
fn main() {
    let r = Runtime(30);
    println!("{}", serde_json::to_string(&r).unwrap());

    println!("{:?}", serde_json::from_str::<Runtime>("\"180 mins\"").unwrap());

    let w = W {
        a:  104,   b:  "hello".to_owned(),  c: r,
    };

    println!("{}", serde_json::to_string(&w).unwrap());

    let input = r#"{"a":104,"b":"hello","c":"180 mins"}"#;
    println!("{:?}", serde_json::from_str::<W>(input).unwrap())
   
}
*/
