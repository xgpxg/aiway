use serde::{Deserialize, Serialize};

/*fn main() {
    #[derive(Debug, Serialize, Deserialize)]
    struct A {
        name: String,
        #[serde(skip_serializing)]
        age: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct B {
        #[serde(flatten)]
        inner: A,
        sex: String,
    }

    let a = A {
        name: "Tom".to_string(),
        age: 18,
    };
    let b = B {
        inner: a,
        sex: "boy".to_string(),
    };

    println!("{}", serde_json::to_string(&b).unwrap());
    println!("{:?}", serde_json::from_str::<B>(r#"{"name":"Tom","sex":"boy"}"#).unwrap());
}
*/

fn main() {
    #[derive(Debug, Serialize, Deserialize)]
    struct A {
        name: String,
        #[serde(skip_serializing_if = "skip")]
        sex: Sex,
    }

    fn skip(_: &Sex) -> bool {
        true
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "other")]
    enum Sex {
        Boy(Job),
        Girl,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Job {
        job: String,
    }

    let a = A {
        name: "Tom".to_string(),
        sex: Sex::Boy(Job {
            job: "worker".to_string(),
        }),
    };

    println!("{}", serde_json::to_string(&a).unwrap());
}
